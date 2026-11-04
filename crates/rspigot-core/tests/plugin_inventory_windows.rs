mod common;
#[path = "common/screen.rs"]
mod screen;

use common::*;
use rspigot_core::events::{ClickInput, ClientEvent, PlayerInput};
use rspigot_core::plugins::events::{
    ClickKind, DragType, EventResult, InventoryAction, InventoryClose, InventoryCreative,
    InventoryCreativePatch, InventoryDrag, InventoryDragPatch, InventoryOpen,
    InventoryTypeSlotType, PrepareItemCraft, PrepareItemCraftPatch, PrepareSmithing,
    PrepareSmithingPatch, SmithItemPatch,
};
use rspigot_core::plugins::{
    Answer, EventKind, EventPatch, EventPayload, HandlerSpec, InventoryOwner, InventoryState,
    Order, Phase, PlayerOrder, Question, ServerOrder,
};
use rspigot_domain::BlockPos;
use rspigot_domain::player::GameMode;
use screen::*;

const CHEST: BlockPos = BlockPos::new(2, 64, 0);
const TABLE: BlockPos = BlockPos::new(-2, 64, 0);
const SMITHY: BlockPos = BlockPos::new(0, 64, 2);
const MENU: u64 = 4;
const STONE: &str = "minecraft:stone";
const LOG: &str = "minecraft:oak_log";

fn inventory(s: &mut Scene, player: &Player) -> InventoryState {
    match s.ask(Question::Inventory(player.id)) {
        Answer::Inventory(state) => state,
        other => panic!("expected an inventory, got {other:?}"),
    }
}

fn held(s: &mut Scene, player: &Player, slot: usize) -> Option<(String, i32)> {
    inventory(s, player).slots[slot]
        .as_ref()
        .map(|item| (item.key.clone(), item.count))
}

fn count_of(s: &mut Scene, player: &Player, key: &str) -> i32 {
    inventory(s, player)
        .slots
        .iter()
        .flatten()
        .filter(|item| item.key == key)
        .map(|item| item.count)
        .sum()
}

fn pickup(view: &mut View, slot: i16, taken: (&str, u8)) -> rspigot_core::events::ContainerClick {
    predict(
        view,
        press(slot, 0, ClickInput::Pickup),
        &[(slot, None)],
        stack(taken.0, taken.1),
    )
}

fn place(view: &mut View, slot: i16, placed: (&str, u8)) -> rspigot_core::events::ContainerClick {
    predict(
        view,
        press(slot, 0, ClickInput::Pickup),
        &[(slot, stack(placed.0, placed.1))],
        None,
    )
}

fn carry(s: &mut Scene, player: &Player, view: &mut View, from: i16, item: (&str, u8)) {
    let take = pickup(view, from, item);
    send_click(s, player, view, take);
}

fn drag_of(event: &Fired) -> InventoryDrag {
    match &event.payload {
        EventPayload::InventoryDrag(drag) => drag.clone(),
        other => panic!("expected a drag, got {other:?}"),
    }
}

fn spread(s: &mut Scene, player: &Player, view: &mut View, slots: &[i16], each: u8) {
    send_click(s, player, view, press(-999, 0, ClickInput::QuickCraft));
    for slot in slots {
        send_click(s, player, view, press(*slot, 1, ClickInput::QuickCraft));
    }
    let changed: Vec<(i16, rspigot_domain::item::Slot)> = slots
        .iter()
        .map(|slot| (*slot, stack(STONE, each)))
        .collect();
    let end = predict(view, press(-999, 2, ClickInput::QuickCraft), &changed, None);
    send_click(s, player, view, end);
}

#[test]
fn a_drag_asks_its_end_only_and_a_refusal_keeps_the_slots_and_the_old_cursor() {
    let mut s = scene_with(
        &["Alice"],
        vec![
            deciding(1, EventKind::InventoryDrag),
            observe(2, EventKind::InventoryDrag),
        ],
    );
    let alice = s.players[0].clone();
    give(&mut s, &alice, 20, STONE, 6);
    let mut own = View::inventory();
    refresh(&alice, &mut own);
    carry(&mut s, &alice, &mut own, 20, (STONE, 6));
    spread(&mut s, &alice, &mut own, &[9, 10, 11], 2);
    let asked = asked_for(&s.plugin, EventKind::InventoryDrag);
    assert_eq!(asked.len(), 1);
    let drag = drag_of(&asked[0]);
    assert_eq!(drag.raw_slots, [9, 10, 11]);
    assert_eq!(drag.inventory_slots, [9, 10, 11]);
    assert_eq!(
        drag.new_items,
        [
            (9, spec(STONE, 2)),
            (10, spec(STONE, 2)),
            (11, spec(STONE, 2))
        ]
    );
    assert_eq!(drag.old_cursor, spec(STONE, 6));
    assert_eq!(drag.cursor, None);
    assert_eq!(drag.drag_type, DragType::Even);
    assert_eq!(drag.inventory.kind, "CRAFTING");
    rule_on(
        &mut s.server,
        1,
        asked[0].id,
        true,
        Some(EventPatch::InventoryDrag(InventoryDragPatch {
            cursor: Some(Some(spec("minecraft:diamond", 1))),
            ..InventoryDragPatch::default()
        })),
    );
    let events = alice.link.take();
    assert!(
        events
            .iter()
            .any(|event| matches!(event, ClientEvent::InventoryContent { .. }))
    );
    own.apply(&events);
    assert_eq!(own.at(9), None);
    assert_eq!(own.at(10), None);
    assert_eq!(own.held(), Some((STONE, 6)));
    let settled = settled_for(&s.plugin, EventKind::InventoryDrag);
    assert!(settled[0].cancelled);
    assert_eq!(drag_of(&settled[0]).cursor, None);

    spread(&mut s, &alice, &mut own, &[9, 10], 2);
    let asked = last_asked(&s.plugin, EventKind::InventoryDrag);
    rule_on(
        &mut s.server,
        1,
        asked.id,
        false,
        Some(EventPatch::InventoryDrag(InventoryDragPatch {
            cursor: Some(Some(spec(STONE, 1))),
            ..InventoryDragPatch::default()
        })),
    );
    refresh(&alice, &mut own);
    assert_eq!(own.at(9), Some((STONE, 3)));
    assert_eq!(own.at(10), Some((STONE, 3)));
    assert_eq!(own.held(), Some((STONE, 1)));
    assert_eq!(held(&mut s, &alice, 9), Some((STONE.to_owned(), 3)));
}

#[test]
fn drag_packets_wait_behind_a_parked_click_and_a_plain_click_ends_a_drag_without_an_event() {
    let mut s = scene_with(
        &["Alice"],
        vec![
            deciding(1, EventKind::InventoryClick),
            observe(2, EventKind::InventoryDrag),
            observe(3, EventKind::InventoryClick),
        ],
    );
    let alice = s.players[0].clone();
    give(&mut s, &alice, 20, STONE, 4);
    let mut own = View::inventory();
    refresh(&alice, &mut own);
    carry(&mut s, &alice, &mut own, 20, (STONE, 4));
    let parked = last_asked(&s.plugin, EventKind::InventoryClick);
    spread(&mut s, &alice, &mut own, &[9, 10], 2);
    assert!(settled_for(&s.plugin, EventKind::InventoryDrag).is_empty());
    rule_on(&mut s.server, 1, parked.id, false, None);
    let drags = settled_for(&s.plugin, EventKind::InventoryDrag);
    assert_eq!(drags.len(), 1);
    assert_eq!(drag_of(&drags[0]).raw_slots, [9, 10]);
    refresh(&alice, &mut own);
    assert_eq!(own.at(9), Some((STONE, 2)));
    assert_eq!(own.held(), None);

    give(&mut s, &alice, 20, STONE, 4);
    refresh(&alice, &mut own);
    carry(&mut s, &alice, &mut own, 20, (STONE, 4));
    rule_on(
        &mut s.server,
        1,
        last_asked(&s.plugin, EventKind::InventoryClick).id,
        false,
        None,
    );
    let clicks = settled_for(&s.plugin, EventKind::InventoryClick).len();
    send_click(
        &mut s,
        &alice,
        &mut own,
        press(-999, 0, ClickInput::QuickCraft),
    );
    send_click(
        &mut s,
        &alice,
        &mut own,
        press(11, 1, ClickInput::QuickCraft),
    );
    send_click(&mut s, &alice, &mut own, press(12, 0, ClickInput::Pickup));
    send_click(
        &mut s,
        &alice,
        &mut own,
        press(-999, 2, ClickInput::QuickCraft),
    );
    assert_eq!(
        settled_for(&s.plugin, EventKind::InventoryClick).len(),
        clicks
    );
    assert_eq!(
        asked_for(&s.plugin, EventKind::InventoryClick).len(),
        clicks
    );
    assert_eq!(settled_for(&s.plugin, EventKind::InventoryDrag).len(), 1);
    assert_eq!(held(&mut s, &alice, 11), None);
    assert_eq!(held(&mut s, &alice, 12), None);
}

fn creative(slot: i16, item: Option<(&str, u8)>) -> PlayerInput {
    PlayerInput::CreativeSlot {
        slot,
        stack: item.and_then(|(key, count)| stack(key, count)),
    }
}

fn creative_of(event: &Fired) -> InventoryCreative {
    match &event.payload {
        EventPayload::InventoryCreative(event) => event.clone(),
        other => panic!("expected a creative click, got {other:?}"),
    }
}

#[test]
fn a_creative_slot_is_asked_and_its_cursor_and_result_decide_what_is_placed() {
    let mut s = scene_with(
        &["Alice"],
        vec![
            deciding(1, EventKind::InventoryCreative),
            observe(2, EventKind::InventoryClick),
        ],
    );
    let alice = s.players[0].clone();
    s.act(to(&alice, PlayerOrder::SetGameMode(GameMode::Creative)));
    alice.link.take();
    send_input(
        &mut s.server,
        &alice,
        creative(36, Some(("minecraft:diamond", 1))),
    );
    let asked = last_asked(&s.plugin, EventKind::InventoryCreative);
    let event = creative_of(&asked);
    assert_eq!(event.click, ClickKind::Creative);
    assert_eq!(event.action, InventoryAction::PlaceAll);
    assert_eq!(event.raw_slot, 36);
    assert_eq!(event.slot, 0);
    assert_eq!(event.slot_type, InventoryTypeSlotType::Quickbar);
    assert_eq!(event.cursor, Some(spec("minecraft:diamond", 1)));
    assert_eq!(event.current_item, None);
    assert!(event.left_click);
    assert_eq!(held(&mut s, &alice, 0), None);
    rule_on(&mut s.server, 1, asked.id, true, None);
    assert_eq!(held(&mut s, &alice, 0), None);
    assert!(
        alice
            .link
            .take()
            .iter()
            .any(|event| matches!(event, ClientEvent::InventoryContent { .. }))
    );
    let settled = settled_for(&s.plugin, EventKind::InventoryCreative);
    assert!(settled[0].cancelled);
    assert_eq!(settled.len(), 1);

    send_input(
        &mut s.server,
        &alice,
        creative(36, Some(("minecraft:diamond", 1))),
    );
    rule_on(
        &mut s.server,
        1,
        last_asked(&s.plugin, EventKind::InventoryCreative).id,
        false,
        Some(EventPatch::InventoryCreative(InventoryCreativePatch {
            cursor: Some(Some(spec("minecraft:emerald", 2))),
            ..InventoryCreativePatch::default()
        })),
    );
    assert_eq!(
        held(&mut s, &alice, 0),
        Some(("minecraft:emerald".to_owned(), 2))
    );

    let asked = asked_for(&s.plugin, EventKind::InventoryCreative).len();
    send_input(
        &mut s.server,
        &alice,
        creative(36, Some(("minecraft:emerald", 2))),
    );
    assert_eq!(
        asked_for(&s.plugin, EventKind::InventoryCreative).len(),
        asked
    );

    send_input(
        &mut s.server,
        &alice,
        creative(37, Some(("minecraft:diamond", 99))),
    );
    rule_on(
        &mut s.server,
        1,
        last_asked(&s.plugin, EventKind::InventoryCreative).id,
        false,
        None,
    );
    assert_eq!(held(&mut s, &alice, 1), None);
    send_input(
        &mut s.server,
        &alice,
        creative(37, Some(("minecraft:diamond", 99))),
    );
    rule_on(
        &mut s.server,
        1,
        last_asked(&s.plugin, EventKind::InventoryCreative).id,
        false,
        Some(EventPatch::InventoryCreative(InventoryCreativePatch {
            result: Some(EventResult::Allow),
            ..InventoryCreativePatch::default()
        })),
    );
    assert_eq!(
        held(&mut s, &alice, 1),
        Some(("minecraft:diamond".to_owned(), 99))
    );

    send_input(
        &mut s.server,
        &alice,
        creative(-1, Some(("minecraft:stone", 3))),
    );
    let asked = last_asked(&s.plugin, EventKind::InventoryCreative);
    let dropped = creative_of(&asked);
    assert_eq!(dropped.raw_slot, -999);
    assert_eq!(dropped.slot_type, InventoryTypeSlotType::Outside);
    assert_eq!(dropped.clicked_inventory, None);
    assert!(s.server.ground_items().is_empty());
    rule_on(&mut s.server, 1, asked.id, false, None);
    assert_eq!(s.server.ground_items().len(), 1);
    let heard = settled_for(&s.plugin, EventKind::InventoryCreative);
    assert!(
        heard
            .iter()
            .all(|event| event.payload.kind() == EventKind::InventoryCreative)
    );
}

fn opens(s: &Scene) -> Vec<InventoryOpen> {
    fired(&s.plugin)
        .into_iter()
        .filter(|event| event.phase == Phase::Settled)
        .filter_map(|event| match event.payload {
            EventPayload::InventoryOpen(open) => Some(open),
            _ => None,
        })
        .collect()
}

fn closes(s: &Scene) -> Vec<InventoryClose> {
    fired(&s.plugin)
        .into_iter()
        .filter(|event| event.phase == Phase::Settled)
        .filter_map(|event| match event.payload {
            EventPayload::InventoryClose(close) => Some(close),
            _ => None,
        })
        .collect()
}

fn window_events(s: &Scene) -> Vec<(EventKind, String)> {
    fired(&s.plugin)
        .into_iter()
        .filter(|event| event.phase == Phase::Settled)
        .filter_map(|event| match event.payload {
            EventPayload::InventoryOpen(open) => {
                Some((EventKind::InventoryOpen, open.inventory.kind))
            }
            EventPayload::InventoryClose(close) => {
                Some((EventKind::InventoryClose, close.inventory.kind))
            }
            _ => None,
        })
        .collect()
}

fn window_handlers() -> Vec<HandlerSpec> {
    vec![
        observe(1, EventKind::InventoryOpen),
        observe(2, EventKind::InventoryClose),
    ]
}

#[test]
fn opening_and_closing_every_window_fires_its_events_in_spigot_order() {
    let mut s = scene_with(&["Alice"], window_handlers());
    let alice = s.players[0].clone();
    set_block(&mut s, CHEST, "minecraft:chest");
    set_block(&mut s, TABLE, "minecraft:crafting_table");
    set_block(&mut s, SMITHY, "minecraft:smithing_table");
    set_block(&mut s, BlockPos::new(0, 64, -2), "minecraft:furnace");
    alice.link.take();
    let chest = open_block(&mut s, &alice, CHEST);
    let opened = opens(&s);
    assert_eq!(opened.len(), 1);
    assert_eq!(opened[0].inventory.kind, "CHEST");
    assert_eq!(opened[0].inventory.owner, InventoryOwner::Block(CHEST));
    assert_eq!(opened[0].view.title, "Chest");
    assert_eq!(opened[0].viewers.len(), 1);
    assert_eq!(opened[0].player.name, "Alice");
    let table = open_block(&mut s, &alice, TABLE);
    assert!(table.window != chest.window);
    let _smithy = open_block(&mut s, &alice, SMITHY);
    let mut furnace = open_block(&mut s, &alice, BlockPos::new(0, 64, -2));
    let mut own = View::inventory();
    close_window(&mut s, &alice, &mut furnace);
    close_window(&mut s, &alice, &mut own);
    assert_eq!(
        window_events(&s),
        [
            (EventKind::InventoryOpen, "CHEST".to_owned()),
            (EventKind::InventoryClose, "CHEST".to_owned()),
            (EventKind::InventoryOpen, "WORKBENCH".to_owned()),
            (EventKind::InventoryClose, "WORKBENCH".to_owned()),
            (EventKind::InventoryOpen, "SMITHING".to_owned()),
            (EventKind::InventoryClose, "SMITHING".to_owned()),
            (EventKind::InventoryOpen, "FURNACE".to_owned()),
            (EventKind::InventoryClose, "FURNACE".to_owned()),
            (EventKind::InventoryClose, "CRAFTING".to_owned()),
        ]
    );
    let closed = closes(&s);
    assert!(closed[..3].iter().all(|close| !close.by_player));
    assert!(closed[3].by_player);
    assert_eq!(closed[3].viewers.len(), 1);
    assert!(closed[4].by_player);
    assert_eq!(closed[4].inventory.owner, InventoryOwner::Player(alice.id));

    s.act(Order::Server(ServerOrder::CreateInventory {
        id: MENU,
        rows: 2,
        title: "Shop".to_owned(),
        locked: true,
    }));
    s.act(to(&alice, PlayerOrder::OpenInventory(MENU)));
    let mut menu = opened_view(&alice).expect("the plugin menu opens");
    let shop = opens(&s).pop().expect("a plugin menu open");
    assert_eq!(
        shop.inventory.owner,
        InventoryOwner::Custom {
            plugin: "tester".to_owned(),
            id: MENU
        }
    );
    assert_eq!(shop.view.title, "Shop");
    assert_eq!(shop.viewers.len(), 1);
    close_window(&mut s, &alice, &mut menu);
    let close = closes(&s).pop().expect("a plugin menu close");
    assert_eq!(
        close.inventory.owner,
        InventoryOwner::Custom {
            plugin: "tester".to_owned(),
            id: MENU
        }
    );
    assert!(close.by_player);
    assert!(s.warnings().is_empty());
}

#[test]
fn a_cancelled_open_shows_no_screen_after_closing_the_old_one() {
    let mut s = scene_with(
        &["Alice"],
        vec![
            deciding(1, EventKind::InventoryOpen),
            observe(2, EventKind::InventoryClose),
            observe(3, EventKind::InventoryOpen),
        ],
    );
    let alice = s.players[0].clone();
    set_block(&mut s, CHEST, "minecraft:chest");
    set_block(&mut s, TABLE, "minecraft:crafting_table");
    alice.link.take();
    use_block(&mut s, &alice, CHEST);
    assert!(opened_view(&alice).is_none());
    let asked = last_asked(&s.plugin, EventKind::InventoryOpen);
    rule_on(&mut s.server, 1, asked.id, false, None);
    let mut chest = opened_view(&alice).expect("the chest opens once the event settles");
    use_block(&mut s, &alice, TABLE);
    let events = alice.link.take();
    assert!(events.contains(&ClientEvent::ContainerClose(chest.window)));
    chest.apply(&events);
    let asked = last_asked(&s.plugin, EventKind::InventoryOpen);
    rule_on(&mut s.server, 1, asked.id, true, None);
    assert!(
        !alice
            .link
            .take()
            .iter()
            .any(|event| matches!(event, ClientEvent::OpenScreen { .. }))
    );
    assert_eq!(
        window_events(&s),
        [
            (EventKind::InventoryOpen, "CHEST".to_owned()),
            (EventKind::InventoryClose, "CHEST".to_owned()),
            (EventKind::InventoryOpen, "WORKBENCH".to_owned()),
        ]
    );
    let settled = settled_for(&s.plugin, EventKind::InventoryOpen);
    assert!(settled.last().expect("settled").cancelled);
    give(&mut s, &alice, 9, STONE, 1);
    let mut own = View::inventory();
    refresh(&alice, &mut own);
    carry(&mut s, &alice, &mut own, 9, (STONE, 1));
    assert_eq!(own.held(), Some((STONE, 1)));
}

fn prepared(s: &Scene) -> Vec<PrepareItemCraft> {
    fired(&s.plugin)
        .into_iter()
        .filter(|event| event.phase == Phase::Settled)
        .filter_map(|event| match event.payload {
            EventPayload::PrepareItemCraft(prepare) => Some(prepare),
            _ => None,
        })
        .collect()
}

#[test]
fn a_grid_change_prepares_the_craft_and_a_patch_sets_the_result() {
    let mut s = scene_with(
        &["Alice"],
        vec![
            deciding(1, EventKind::PrepareItemCraft),
            observe(2, EventKind::PrepareItemCraft),
        ],
    );
    let alice = s.players[0].clone();
    give(&mut s, &alice, 9, LOG, 1);
    let mut own = View::inventory();
    refresh(&alice, &mut own);
    carry(&mut s, &alice, &mut own, 9, (LOG, 1));
    let put = place(&mut own, 1, (LOG, 1));
    send_click(&mut s, &alice, &mut own, put);
    let asked = last_asked(&s.plugin, EventKind::PrepareItemCraft);
    let EventPayload::PrepareItemCraft(event) = &asked.payload else {
        panic!("a prepare is asked");
    };
    assert_eq!(event.recipe.as_deref(), Some("minecraft:oak_planks"));
    assert!(!event.repair);
    assert_eq!(event.result, Some(spec("minecraft:oak_planks", 4)));
    assert_eq!(event.inventory.kind, "CRAFTING");
    rule_on(
        &mut s.server,
        1,
        asked.id,
        false,
        Some(EventPatch::PrepareItemCraft(PrepareItemCraftPatch {
            result: Some(Some(spec("minecraft:diamond", 2))),
        })),
    );
    refresh(&alice, &mut own);
    assert_eq!(own.at(0), Some(("minecraft:diamond", 2)));
    assert_eq!(
        prepared(&s).last().and_then(|event| event.result.clone()),
        Some(spec("minecraft:diamond", 2))
    );
    let take = pickup(&mut own, 1, (LOG, 1));
    send_click(&mut s, &alice, &mut own, take);
    let asked = last_asked(&s.plugin, EventKind::PrepareItemCraft);
    let EventPayload::PrepareItemCraft(event) = &asked.payload else {
        panic!("a prepare is asked");
    };
    assert_eq!(event.recipe, None);
    assert_eq!(event.result, None);
    rule_on(&mut s.server, 1, asked.id, false, None);
    refresh(&alice, &mut own);
    assert_eq!(own.at(0), None);
}

#[test]
fn a_repair_is_prepared_as_one() {
    let mut s = scene_with(&["Alice"], vec![observe(2, EventKind::PrepareItemCraft)]);
    let alice = s.players[0].clone();
    give(&mut s, &alice, 9, "minecraft:iron_sword", 1);
    give(&mut s, &alice, 10, "minecraft:iron_sword", 1);
    let mut own = View::inventory();
    refresh(&alice, &mut own);
    carry(&mut s, &alice, &mut own, 9, ("minecraft:iron_sword", 1));
    let put = place(&mut own, 1, ("minecraft:iron_sword", 1));
    send_click(&mut s, &alice, &mut own, put);
    carry(&mut s, &alice, &mut own, 10, ("minecraft:iron_sword", 1));
    let put = place(&mut own, 4, ("minecraft:iron_sword", 1));
    send_click(&mut s, &alice, &mut own, put);
    let last = prepared(&s).pop().expect("prepared");
    assert!(last.repair);
    assert_eq!(last.recipe.as_deref(), Some("minecraft:repair_item"));
}

#[test]
fn a_queued_shift_click_reads_the_patched_result_and_keeps_crafting_it() {
    let mut s = scene_with(
        &["Alice"],
        vec![
            deciding(1, EventKind::PrepareItemCraft),
            observe(2, EventKind::CraftItem),
        ],
    );
    let alice = s.players[0].clone();
    set_block(&mut s, TABLE, "minecraft:crafting_table");
    give(&mut s, &alice, 0, LOG, 2);
    alice.link.take();
    let mut table = open_block(&mut s, &alice, TABLE);
    carry(&mut s, &alice, &mut table, 37, (LOG, 2));
    let put = place(&mut table, 1, (LOG, 2));
    send_click(&mut s, &alice, &mut table, put);
    let asked = last_asked(&s.plugin, EventKind::PrepareItemCraft);
    send_click(
        &mut s,
        &alice,
        &mut table,
        press(0, 0, ClickInput::QuickMove),
    );
    assert!(settled_for(&s.plugin, EventKind::CraftItem).is_empty());
    let diamond = || {
        Some(EventPatch::PrepareItemCraft(PrepareItemCraftPatch {
            result: Some(Some(spec("minecraft:diamond", 1))),
        }))
    };
    rule_on(&mut s.server, 1, asked.id, false, diamond());
    let crafts = settled_for(&s.plugin, EventKind::CraftItem);
    assert_eq!(crafts.len(), 1);
    let EventPayload::CraftItem(craft) = &crafts[0].payload else {
        panic!("a craft");
    };
    assert_eq!(craft.current_item, Some(spec("minecraft:diamond", 1)));
    assert_eq!(craft.action, InventoryAction::MoveToOtherInventory);
    let between = last_asked(&s.plugin, EventKind::PrepareItemCraft);
    assert_ne!(between.id, asked.id);
    rule_on(&mut s.server, 1, between.id, false, diamond());
    let last = last_asked(&s.plugin, EventKind::PrepareItemCraft);
    let EventPayload::PrepareItemCraft(event) = &last.payload else {
        panic!("a prepare");
    };
    assert_eq!(event.recipe, None);
    rule_on(&mut s.server, 1, last.id, false, None);
    assert_eq!(count_of(&mut s, &alice, "minecraft:diamond"), 2);
    assert_eq!(count_of(&mut s, &alice, LOG), 0);
    assert_eq!(count_of(&mut s, &alice, "minecraft:oak_planks"), 0);
    assert_eq!(settled_for(&s.plugin, EventKind::CraftItem).len(), 1);
}

#[test]
fn three_smithing_inputs_prepare_the_result_and_a_patch_replaces_it() {
    let mut s = scene_with(
        &["Alice"],
        vec![
            deciding(1, EventKind::PrepareSmithing),
            observe(2, EventKind::SmithItem),
        ],
    );
    let alice = s.players[0].clone();
    set_block(&mut s, SMITHY, "minecraft:smithing_table");
    give(
        &mut s,
        &alice,
        0,
        "minecraft:netherite_upgrade_smithing_template",
        1,
    );
    give(&mut s, &alice, 1, "minecraft:diamond_sword", 1);
    give(&mut s, &alice, 2, "minecraft:netherite_ingot", 1);
    alice.link.take();
    let mut smithy = open_block(&mut s, &alice, SMITHY);
    for (index, key) in [
        "minecraft:netherite_upgrade_smithing_template",
        "minecraft:diamond_sword",
        "minecraft:netherite_ingot",
    ]
    .into_iter()
    .enumerate()
    {
        let hotbar = 31 + i16::try_from(index).expect("small");
        carry(&mut s, &alice, &mut smithy, hotbar, (key, 1));
        let put = place(&mut smithy, i16::try_from(index).expect("small"), (key, 1));
        send_click(&mut s, &alice, &mut smithy, put);
    }
    let asked = asked_for(&s.plugin, EventKind::PrepareSmithing);
    assert_eq!(asked.len(), 1);
    let EventPayload::PrepareSmithing(PrepareSmithing {
        result, inventory, ..
    }) = &asked[0].payload
    else {
        panic!("a smithing prepare");
    };
    assert_eq!(result, &Some(spec("minecraft:netherite_sword", 1)));
    assert_eq!(inventory.kind, "SMITHING");
    rule_on(
        &mut s.server,
        1,
        asked[0].id,
        false,
        Some(EventPatch::PrepareSmithing(PrepareSmithingPatch {
            result: Some(Some(spec("minecraft:golden_sword", 1))),
        })),
    );
    refresh(&alice, &mut smithy);
    assert_eq!(smithy.at(3), Some(("minecraft:golden_sword", 1)));
    let take = pickup(&mut smithy, 3, ("minecraft:golden_sword", 1));
    send_click(&mut s, &alice, &mut smithy, take);
    let smiths = settled_for(&s.plugin, EventKind::SmithItem);
    assert_eq!(smiths.len(), 1);
    refresh(&alice, &mut smithy);
    assert_eq!(smithy.held(), Some(("minecraft:golden_sword", 1)));
    assert_eq!(smithy.at(0), None);
}

#[test]
fn a_cancelled_smith_keeps_the_inputs() {
    let mut s = scene_with(
        &["Alice"],
        vec![
            deciding(1, EventKind::SmithItem),
            observe(2, EventKind::SmithItem),
        ],
    );
    let alice = s.players[0].clone();
    set_block(&mut s, SMITHY, "minecraft:smithing_table");
    give(
        &mut s,
        &alice,
        0,
        "minecraft:netherite_upgrade_smithing_template",
        1,
    );
    give(&mut s, &alice, 1, "minecraft:diamond_sword", 1);
    give(&mut s, &alice, 2, "minecraft:netherite_ingot", 1);
    alice.link.take();
    let mut smithy = open_block(&mut s, &alice, SMITHY);
    for (index, key) in [
        "minecraft:netherite_upgrade_smithing_template",
        "minecraft:diamond_sword",
        "minecraft:netherite_ingot",
    ]
    .into_iter()
    .enumerate()
    {
        let hotbar = 31 + i16::try_from(index).expect("small");
        carry(&mut s, &alice, &mut smithy, hotbar, (key, 1));
        let put = place(&mut smithy, i16::try_from(index).expect("small"), (key, 1));
        send_click(&mut s, &alice, &mut smithy, put);
    }
    let take = pickup(&mut smithy, 3, ("minecraft:netherite_sword", 1));
    send_click(&mut s, &alice, &mut smithy, take);
    let asked = last_asked(&s.plugin, EventKind::SmithItem);
    rule_on(&mut s.server, 1, asked.id, true, None);
    let events = alice.link.take();
    assert!(events.iter().any(|event| matches!(
        event,
        ClientEvent::ContainerContent { window, .. } if *window == smithy.window
    )));
    smithy.apply(&events);
    assert_eq!(smithy.at(1), Some(("minecraft:diamond_sword", 1)));
    assert_eq!(smithy.at(3), Some(("minecraft:netherite_sword", 1)));
    assert_eq!(smithy.held(), None);
    assert!(settled_for(&s.plugin, EventKind::SmithItem)[0].cancelled);
}

#[test]
fn a_leaving_player_closes_the_open_window_before_leaving() {
    let mut s = scene_with(&["Alice"], window_handlers());
    let alice = s.players[0].clone();
    set_block(&mut s, CHEST, "minecraft:chest");
    alice.link.take();
    let _chest = open_block(&mut s, &alice, CHEST);
    s.server.handle(leave_of(&alice));
    s.server.deliver();
    let closed = closes(&s);
    assert_eq!(closed.len(), 1);
    assert!(!closed[0].by_player);
}

#[test]
fn a_result_set_on_a_grid_without_a_recipe_uses_the_grid_and_stops_when_it_empties() {
    let mut s = scene_with(
        &["Alice"],
        vec![
            deciding(1, EventKind::PrepareItemCraft),
            observe(2, EventKind::CraftItem),
            observe(3, EventKind::PrepareItemCraft),
        ],
    );
    let alice = s.players[0].clone();
    let diamond = || {
        Some(EventPatch::PrepareItemCraft(PrepareItemCraftPatch {
            result: Some(Some(spec("minecraft:diamond", 1))),
        }))
    };
    give(&mut s, &alice, 9, "minecraft:dirt", 1);
    let mut own = View::inventory();
    refresh(&alice, &mut own);
    carry(&mut s, &alice, &mut own, 9, ("minecraft:dirt", 1));
    let put = place(&mut own, 1, ("minecraft:dirt", 1));
    send_click(&mut s, &alice, &mut own, put);
    rule_on(
        &mut s.server,
        1,
        last_asked(&s.plugin, EventKind::PrepareItemCraft).id,
        false,
        diamond(),
    );
    refresh(&alice, &mut own);
    assert_eq!(own.at(0), Some(("minecraft:diamond", 1)));
    let take = pickup(&mut own, 0, ("minecraft:diamond", 1));
    send_click(&mut s, &alice, &mut own, take);
    rule_on(
        &mut s.server,
        1,
        last_asked(&s.plugin, EventKind::PrepareItemCraft).id,
        false,
        None,
    );
    refresh(&alice, &mut own);
    assert_eq!(own.held(), Some(("minecraft:diamond", 1)));
    assert_eq!(own.at(1), None);
    assert_eq!(count_of(&mut s, &alice, "minecraft:dirt"), 0);
    let mut own = View::inventory();
    let close = rspigot_core::events::Inbound::Input {
        player: alice.id,
        input: PlayerInput::ContainerClose(0),
    };
    s.server.handle(close);
    s.server.deliver();
    refresh(&alice, &mut own);

    give(&mut s, &alice, 9, "minecraft:dirt", 2);
    refresh(&alice, &mut own);
    carry(&mut s, &alice, &mut own, 9, ("minecraft:dirt", 2));
    let put = place(&mut own, 1, ("minecraft:dirt", 2));
    send_click(&mut s, &alice, &mut own, put);
    rule_on(
        &mut s.server,
        1,
        last_asked(&s.plugin, EventKind::PrepareItemCraft).id,
        false,
        diamond(),
    );
    send_click(&mut s, &alice, &mut own, press(0, 0, ClickInput::QuickMove));
    for _ in 0..4 {
        let asked = asked_for(&s.plugin, EventKind::PrepareItemCraft);
        let last = asked.last().expect("a prepare");
        if settled_for(&s.plugin, EventKind::PrepareItemCraft)
            .iter()
            .any(|settled| settled.id == last.id)
        {
            break;
        }
        rule_on(&mut s.server, 1, last.id, false, diamond());
    }
    assert_eq!(count_of(&mut s, &alice, "minecraft:dirt"), 0);
    assert_eq!(count_of(&mut s, &alice, "minecraft:diamond"), 3);
}

#[test]
fn vanilla_inventories_carry_a_runtime_id_per_opening() {
    let mut s = scene_with(&["Alice", "Bob"], window_handlers());
    let (alice, bob) = (s.players[0].clone(), s.players[1].clone());
    set_block(&mut s, CHEST, "minecraft:chest");
    set_block(&mut s, TABLE, "minecraft:crafting_table");
    set_block(&mut s, SMITHY, "minecraft:ender_chest");
    alice.link.take();
    bob.link.take();
    let mut alice_table = open_block(&mut s, &alice, TABLE);
    let mut bob_table = open_block(&mut s, &bob, TABLE);
    let tables = opens(&s);
    assert_eq!(tables[0].inventory.owner, tables[1].inventory.owner);
    assert_ne!(tables[0].inventory.id, tables[1].inventory.id);
    assert_ne!(tables[0].inventory.id, 0);
    close_window(&mut s, &alice, &mut alice_table);
    close_window(&mut s, &bob, &mut bob_table);
    let _alice_chest = open_block(&mut s, &alice, CHEST);
    let _bob_chest = open_block(&mut s, &bob, CHEST);
    let chests = opens(&s);
    assert_eq!(chests[2].inventory.id, chests[3].inventory.id);
    let ender = open_block(&mut s, &alice, SMITHY);
    let enders = opens(&s);
    let ender_open = enders.last().expect("the ender chest opens");
    assert_eq!(ender_open.inventory.kind, "ENDER_CHEST");
    assert_eq!(ender_open.inventory.owner, InventoryOwner::Player(alice.id));
    assert_eq!(
        ender_open.view.bottom.owner,
        InventoryOwner::Player(alice.id)
    );
    assert_ne!(ender_open.inventory.id, ender_open.view.bottom.id);
    let mut ender = ender;
    let mut own = View::inventory();
    close_window(&mut s, &alice, &mut ender);
    close_window(&mut s, &alice, &mut own);
    let grid = closes(&s).pop().expect("the player inventory closes");
    assert_eq!(grid.inventory.kind, "CRAFTING");
    assert_ne!(grid.inventory.id, grid.view.bottom.id);
    assert_eq!(grid.view.bottom.id, ender_open.view.bottom.id);
}

fn result_packets(events: &[ClientEvent], window: i32) -> Vec<Option<(&'static str, u8)>> {
    events
        .iter()
        .filter_map(|event| match event {
            ClientEvent::InventorySlot { slot: 0, stack, .. } if window == 0 => Some(shown(stack)),
            ClientEvent::ContainerSlot {
                window: at,
                slot: 0,
                stack,
                ..
            } if *at == window => Some(shown(stack)),
            ClientEvent::InventoryContent { slots, .. } if window == 0 => Some(shown(&slots[0])),
            ClientEvent::ContainerContent {
                window: at, slots, ..
            } if *at == window => Some(shown(&slots[0])),
            _ => None,
        })
        .collect()
}

#[test]
fn the_vanilla_result_stays_hidden_while_a_prepare_decider_answers() {
    let mut s = scene_with(&["Alice"], vec![deciding(1, EventKind::PrepareItemCraft)]);
    let alice = s.players[0].clone();
    give(&mut s, &alice, 9, LOG, 1);
    set_block(&mut s, TABLE, "minecraft:crafting_table");
    give(&mut s, &alice, 0, LOG, 1);
    let mut own = View::inventory();
    refresh(&alice, &mut own);
    carry(&mut s, &alice, &mut own, 9, (LOG, 1));
    let put = predict(
        &mut own,
        press(1, 0, ClickInput::Pickup),
        &[(1, stack(LOG, 1))],
        None,
    );
    s.server.handle(rspigot_core::events::Inbound::Input {
        player: alice.id,
        input: PlayerInput::ContainerClick(rspigot_core::events::ContainerClick {
            container: 0,
            state_id: own.state,
            ..put
        }),
    });
    s.server.deliver();
    s.tick(1);
    let mut seen = alice.link.take();
    let asked = last_asked(&s.plugin, EventKind::PrepareItemCraft);
    rule_on(
        &mut s.server,
        1,
        asked.id,
        false,
        Some(EventPatch::PrepareItemCraft(PrepareItemCraftPatch {
            result: Some(Some(spec("minecraft:diamond", 2))),
        })),
    );
    seen.extend(alice.link.take());
    assert!(!result_packets(&seen, 0).contains(&Some(("minecraft:oak_planks", 4))));
    assert_eq!(
        result_packets(&seen, 0).last(),
        Some(&Some(("minecraft:diamond", 2)))
    );
    let mut own = View::inventory();
    close_window(&mut s, &alice, &mut own);
    alice.link.take();

    let mut table = open_block(&mut s, &alice, TABLE);
    carry(&mut s, &alice, &mut table, 37, (LOG, 2));
    let put = predict(
        &mut table,
        press(1, 0, ClickInput::Pickup),
        &[(1, stack(LOG, 2))],
        None,
    );
    s.server.handle(rspigot_core::events::Inbound::Input {
        player: alice.id,
        input: PlayerInput::ContainerClick(rspigot_core::events::ContainerClick {
            container: table.window,
            state_id: table.state,
            ..put
        }),
    });
    s.server.deliver();
    s.tick(1);
    let mut seen = alice.link.take();
    let asked = last_asked(&s.plugin, EventKind::PrepareItemCraft);
    rule_on(&mut s.server, 1, asked.id, false, None);
    seen.extend(alice.link.take());
    let shown_results = result_packets(&seen, table.window);
    assert_eq!(
        shown_results.iter().filter(|shown| shown.is_some()).count(),
        1,
        "{shown_results:?}"
    );
    assert_eq!(
        shown_results.last(),
        Some(&Some(("minecraft:oak_planks", 4)))
    );
}

#[test]
fn a_cancelled_creative_slot_keeps_its_current_item_write_and_drops_its_cursor() {
    let mut s = scene_with(
        &["Alice"],
        vec![
            deciding(1, EventKind::InventoryCreative),
            observe(2, EventKind::InventoryCreative),
        ],
    );
    let alice = s.players[0].clone();
    s.act(to(&alice, PlayerOrder::SetGameMode(GameMode::Creative)));
    give(&mut s, &alice, 0, STONE, 3);
    alice.link.take();
    send_input(
        &mut s.server,
        &alice,
        creative(36, Some(("minecraft:diamond", 1))),
    );
    rule_on(
        &mut s.server,
        1,
        last_asked(&s.plugin, EventKind::InventoryCreative).id,
        true,
        Some(EventPatch::InventoryCreative(InventoryCreativePatch {
            current_item: Some(Some(spec("minecraft:gold_ingot", 2))),
            cursor: Some(Some(spec("minecraft:emerald", 1))),
            ..InventoryCreativePatch::default()
        })),
    );
    let settled = settled_for(&s.plugin, EventKind::InventoryCreative);
    let event = creative_of(&settled[0]);
    assert!(settled[0].cancelled);
    assert_eq!(event.cursor, Some(spec("minecraft:diamond", 1)));
    assert_eq!(event.current_item, Some(spec("minecraft:gold_ingot", 2)));
    assert_eq!(
        held(&mut s, &alice, 0),
        Some(("minecraft:gold_ingot".to_owned(), 2))
    );
}

const CHEST_HOTBAR: i16 = 54;

#[test]
fn a_drag_in_a_chest_and_in_a_plugin_inventory_is_asked_with_its_raw_and_converted_slots() {
    let mut s = scene_with(
        &["Alice"],
        vec![
            deciding(1, EventKind::InventoryDrag),
            observe(2, EventKind::InventoryDrag),
        ],
    );
    let alice = s.players[0].clone();
    set_block(&mut s, CHEST, "minecraft:chest");
    give(&mut s, &alice, 0, STONE, 6);
    alice.link.take();
    let mut chest = open_block(&mut s, &alice, CHEST);
    carry(&mut s, &alice, &mut chest, CHEST_HOTBAR, (STONE, 6));
    spread(&mut s, &alice, &mut chest, &[0, 1, 27], 2);
    let asked = last_asked(&s.plugin, EventKind::InventoryDrag);
    let drag = drag_of(&asked);
    assert_eq!(drag.inventory.kind, "CHEST");
    assert_eq!(drag.raw_slots, [0, 1, 27]);
    assert_eq!(drag.inventory_slots, [0, 1, 9]);
    rule_on(&mut s.server, 1, asked.id, true, None);
    refresh(&alice, &mut chest);
    assert_eq!(chest.at(0), None);
    assert_eq!(chest.at(27), None);
    assert_eq!(chest.held(), Some((STONE, 6)));
    spread(&mut s, &alice, &mut chest, &[0, 1, 27], 2);
    rule_on(
        &mut s.server,
        1,
        last_asked(&s.plugin, EventKind::InventoryDrag).id,
        false,
        None,
    );
    refresh(&alice, &mut chest);
    assert_eq!(chest.at(0), Some((STONE, 2)));
    assert_eq!(chest.at(27), Some((STONE, 2)));
    assert_eq!(held(&mut s, &alice, 9), Some((STONE.to_owned(), 2)));
    close_window(&mut s, &alice, &mut chest);

    s.act(Order::Server(ServerOrder::CreateInventory {
        id: MENU,
        rows: 1,
        title: "Box".to_owned(),
        locked: false,
    }));
    give(&mut s, &alice, 0, STONE, 4);
    s.act(to(&alice, PlayerOrder::OpenInventory(MENU)));
    let mut menu = opened_view(&alice).expect("the plugin menu opens");
    carry(&mut s, &alice, &mut menu, 36, (STONE, 4));
    spread(&mut s, &alice, &mut menu, &[3, 4], 2);
    let asked = last_asked(&s.plugin, EventKind::InventoryDrag);
    let drag = drag_of(&asked);
    assert_eq!(drag.raw_slots, [3, 4]);
    assert_eq!(drag.inventory_slots, [3, 4]);
    assert_eq!(
        drag.inventory.owner,
        InventoryOwner::Custom {
            plugin: "tester".to_owned(),
            id: MENU
        }
    );
    rule_on(&mut s.server, 1, asked.id, false, None);
    refresh(&alice, &mut menu);
    assert_eq!(menu.at(3), Some((STONE, 2)));
    assert_eq!(menu.at(4), Some((STONE, 2)));
    assert_eq!(menu.held(), None);
}

fn smithy_ready(s: &mut Scene, alice: &Player) -> View {
    set_block(s, SMITHY, "minecraft:smithing_table");
    give(
        s,
        alice,
        0,
        "minecraft:netherite_upgrade_smithing_template",
        1,
    );
    give(s, alice, 1, "minecraft:diamond_sword", 1);
    give(s, alice, 2, "minecraft:netherite_ingot", 1);
    alice.link.take();
    let mut smithy = open_block(s, alice, SMITHY);
    for (index, key) in [
        "minecraft:netherite_upgrade_smithing_template",
        "minecraft:diamond_sword",
        "minecraft:netherite_ingot",
    ]
    .into_iter()
    .enumerate()
    {
        let slot = i16::try_from(index).expect("small");
        carry(s, alice, &mut smithy, 31 + slot, (key, 1));
        let put = place(&mut smithy, slot, (key, 1));
        send_click(s, alice, &mut smithy, put);
    }
    smithy
}

#[test]
fn a_smith_patch_writes_the_taken_item_and_its_result_refuses_it() {
    let mut s = scene_with(
        &["Alice"],
        vec![
            deciding(1, EventKind::SmithItem),
            observe(2, EventKind::SmithItem),
        ],
    );
    let alice = s.players[0].clone();
    let mut smithy = smithy_ready(&mut s, &alice);
    let take = pickup(&mut smithy, 3, ("minecraft:netherite_sword", 1));
    send_click(&mut s, &alice, &mut smithy, take);
    rule_on(
        &mut s.server,
        1,
        last_asked(&s.plugin, EventKind::SmithItem).id,
        false,
        Some(EventPatch::SmithItem(SmithItemPatch {
            current_item: Some(Some(spec("minecraft:golden_sword", 1))),
            ..SmithItemPatch::default()
        })),
    );
    refresh(&alice, &mut smithy);
    assert_eq!(smithy.held(), Some(("minecraft:golden_sword", 1)));
    assert_eq!(smithy.at(1), None);
    close_window(&mut s, &alice, &mut smithy);

    let mut smithy = smithy_ready(&mut s, &alice);
    let take = pickup(&mut smithy, 3, ("minecraft:netherite_sword", 1));
    send_click(&mut s, &alice, &mut smithy, take);
    rule_on(
        &mut s.server,
        1,
        last_asked(&s.plugin, EventKind::SmithItem).id,
        false,
        Some(EventPatch::SmithItem(SmithItemPatch {
            cursor: Some(Some(spec("minecraft:emerald", 4))),
            result: Some(EventResult::Deny),
            ..SmithItemPatch::default()
        })),
    );
    refresh(&alice, &mut smithy);
    assert_eq!(smithy.held(), Some(("minecraft:emerald", 4)));
    assert_eq!(smithy.at(1), Some(("minecraft:diamond_sword", 1)));
    assert_eq!(smithy.at(3), Some(("minecraft:netherite_sword", 1)));
    assert!(
        settled_for(&s.plugin, EventKind::SmithItem)
            .last()
            .expect("settled")
            .cancelled
    );
}

fn parked_chest_click(s: &mut Scene, alice: &Player) -> (View, Fired) {
    set_block(s, CHEST, "minecraft:chest");
    give(s, alice, 0, STONE, 3);
    alice.link.take();
    let mut chest = open_block(s, alice, CHEST);
    carry(s, alice, &mut chest, CHEST_HOTBAR, (STONE, 3));
    rule_on(
        &mut s.server,
        1,
        last_asked(&s.plugin, EventKind::InventoryClick).id,
        false,
        None,
    );
    let put = place(&mut chest, 0, (STONE, 3));
    send_click(s, alice, &mut chest, put);
    rule_on(
        &mut s.server,
        1,
        last_asked(&s.plugin, EventKind::InventoryClick).id,
        false,
        None,
    );
    let take = pickup(&mut chest, 0, (STONE, 3));
    send_click(s, alice, &mut chest, take);
    let parked = last_asked(&s.plugin, EventKind::InventoryClick);
    (chest, parked)
}

#[test]
fn a_parked_click_is_dropped_when_its_window_closed_its_menu_broke_or_its_player_died() {
    let clicks = || {
        vec![
            deciding(1, EventKind::InventoryClick),
            observe(2, EventKind::InventoryClick),
        ]
    };
    let mut s = scene_with(&["Alice"], clicks());
    let alice = s.players[0].clone();
    let (_chest, parked) = parked_chest_click(&mut s, &alice);
    s.act(to(&alice, PlayerOrder::CloseInventory));
    rule_on(&mut s.server, 1, parked.id, false, None);
    let mut reopened = open_block(&mut s, &alice, CHEST);
    assert_eq!(reopened.at(0), Some((STONE, 3)));
    assert_eq!(reopened.held(), None);
    close_window(&mut s, &alice, &mut reopened);

    let mut s = scene_with(&["Alice"], clicks());
    let alice = s.players[0].clone();
    let (_chest, parked) = parked_chest_click(&mut s, &alice);
    set_block(&mut s, CHEST, "minecraft:air");
    rule_on(&mut s.server, 1, parked.id, false, None);
    assert_eq!(count_of(&mut s, &alice, STONE), 0);

    let mut s = scene_with(&["Alice"], clicks());
    let alice = s.players[0].clone();
    give(&mut s, &alice, 9, STONE, 3);
    let mut own = View::inventory();
    refresh(&alice, &mut own);
    let take = pickup(&mut own, 9, (STONE, 3));
    send_click(&mut s, &alice, &mut own, take);
    let parked = last_asked(&s.plugin, EventKind::InventoryClick);
    s.act(to(&alice, PlayerOrder::Kill));
    rule_on(&mut s.server, 1, parked.id, false, None);
    s.tick(1);
    assert_eq!(count_of(&mut s, &alice, STONE), 0);
    assert!(
        settled_for(&s.plugin, EventKind::InventoryClick)
            .last()
            .is_some_and(|event| event.id == parked.id)
    );
}
