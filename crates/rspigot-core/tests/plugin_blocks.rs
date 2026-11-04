mod common;

use common::*;
use rspigot_core::events::{ClientEvent, DigAction, Direction, Inbound, PlayerInput};
use rspigot_core::plugins::events::{
    BlockBreak, BlockBreakPatch, BlockDamage, BlockDamageAbort, BlockDamagePatch, BlockDropItem,
    BlockDropItemPatch,
};
use rspigot_core::plugins::{
    Answer, BlockOrder, EventKind, EventPatch, EventPayload, ItemSpec, Order, Phase, Question,
};
use rspigot_domain::BlockPos;

const DIRT: BlockPos = BlockPos::new(2, 63, 0);

fn console(s: &mut Scene, line: &str) {
    s.server.handle(Inbound::Console(line.to_owned()));
    s.server.deliver();
}

fn set(s: &mut Scene, pos: BlockPos, key: &str) {
    s.act(Order::Block {
        pos,
        order: BlockOrder::Set(key.to_owned()),
    });
}

fn dig(s: &mut Scene, player: &Player, action: DigAction, pos: BlockPos, sequence: i32) {
    send_input(
        &mut s.server,
        player,
        PlayerInput::Dig {
            action,
            pos,
            face: Direction::Up,
            sequence,
        },
    );
}

const DIRT_TICKS: u32 = 10;

fn mine(s: &mut Scene, player: &Player, pos: BlockPos) {
    dig(s, player, DigAction::Start, pos, 1);
    s.tick(DIRT_TICKS);
    dig(s, player, DigAction::Finish, pos, 2);
}

fn stand(s: &mut Scene, player: &Player) {
    send_input(
        &mut s.server,
        player,
        PlayerInput::Move {
            position: None,
            rotation: None,
            on_ground: true,
        },
    );
}

fn kind_at(s: &mut Scene, pos: BlockPos) -> String {
    match s.ask(Question::Block(pos)) {
        Answer::Block(state) => state.kind,
        other => panic!("expected a block, got {other:?}"),
    }
}

fn dirt_scene(handlers: Vec<rspigot_core::plugins::HandlerSpec>) -> (Scene, Player) {
    let mut s = scene_with(&["Alice"], handlers);
    let alice = s.players[0].clone();
    set(&mut s, DIRT, "minecraft:dirt");
    stream_chunk_of(&mut s, DIRT);
    stand(&mut s, &alice);
    alice.link.take();
    (s, alice)
}

fn block_updates_and_acks(events: &[ClientEvent]) -> Vec<String> {
    events
        .iter()
        .filter_map(|event| match event {
            ClientEvent::BlockUpdate { pos, state } => Some(format!(
                "update {} {} {} {}",
                pos.x,
                pos.y,
                pos.z,
                if state.is_air() { "air" } else { "block" }
            )),
            ClientEvent::BlockChangedAck(sequence) => Some(format!("ack {sequence}")),
            _ => None,
        })
        .collect()
}

fn break_of(fired: &Fired) -> BlockBreak {
    match &fired.payload {
        EventPayload::BlockBreak(event) => event.clone(),
        other => panic!("expected a block break, got {other:?}"),
    }
}

fn drop_of(fired: &Fired) -> BlockDropItem {
    match &fired.payload {
        EventPayload::BlockDropItem(event) => event.clone(),
        other => panic!("expected a block drop, got {other:?}"),
    }
}

#[test]
fn mining_a_block_fires_damage_break_and_drop_in_order() {
    let (mut s, alice) = dirt_scene(vec![
        observe(1, EventKind::BlockDamage),
        observe(2, EventKind::BlockBreak),
        observe(3, EventKind::BlockDropItem),
    ]);
    mine(&mut s, &alice, DIRT);
    let kinds: Vec<EventKind> = fired(&s.plugin)
        .into_iter()
        .map(|event| event.payload.kind())
        .collect();
    assert_eq!(
        kinds,
        [
            EventKind::BlockDamage,
            EventKind::BlockBreak,
            EventKind::BlockDropItem
        ]
    );
    let damage = settled_for(&s.plugin, EventKind::BlockDamage);
    assert_eq!(
        damage[0].payload,
        EventPayload::BlockDamage(BlockDamage {
            block: DIRT,
            insta_break: false,
            item_in_hand: ItemSpec::of("minecraft:air", 0),
            player: match &damage[0].payload {
                EventPayload::BlockDamage(event) => event.player.clone(),
                _ => unreachable!(),
            },
        })
    );
    let broke = break_of(&settled_for(&s.plugin, EventKind::BlockBreak)[0]);
    assert_eq!(broke.block, DIRT);
    assert!(broke.drop_items);
    assert_eq!(broke.exp_to_drop, 0);
    assert_eq!(broke.player.name, "Alice");
    let dropped = drop_of(&settled_for(&s.plugin, EventKind::BlockDropItem)[0]);
    assert_eq!(dropped.block, DIRT);
    assert_eq!(dropped.block_state.state.kind, "minecraft:dirt");
    assert_eq!(dropped.items.len(), 1);
    assert_eq!(dropped.items[0].entity.kind, "minecraft:item");
    assert_eq!(dropped.items[0].item, ItemSpec::of("minecraft:dirt", 1));
    assert_eq!(kind_at(&mut s, DIRT), "minecraft:air");
    assert_eq!(s.server.items_on_ground(), 1);
}

#[test]
fn a_finish_without_a_matching_start_is_refused() {
    let (mut s, alice) = dirt_scene(Vec::new());
    dig(&mut s, &alice, DigAction::Finish, DIRT, 5);
    assert_eq!(kind_at(&mut s, DIRT), "minecraft:dirt");
    assert_eq!(
        block_updates_and_acks(&alice.link.take()),
        ["update 2 63 0 block", "ack 5"]
    );
    dig(&mut s, &alice, DigAction::Start, BlockPos::new(3, 63, 0), 6);
    dig(&mut s, &alice, DigAction::Finish, DIRT, 7);
    assert_eq!(kind_at(&mut s, DIRT), "minecraft:dirt");
}

#[test]
fn a_cancelled_break_resends_the_block_its_neighbours_then_acks() {
    let (mut s, alice) = dirt_scene(vec![
        deciding(1, EventKind::BlockBreak),
        observe(2, EventKind::BlockBreak),
    ]);
    dig(&mut s, &alice, DigAction::Start, DIRT, 1);
    s.tick(DIRT_TICKS);
    alice.link.take();
    dig(&mut s, &alice, DigAction::Finish, DIRT, 2);
    assert_eq!(
        block_updates_and_acks(&alice.link.take()),
        ["update 2 63 0 air"]
    );
    let asked = last_asked(&s.plugin, EventKind::BlockBreak);
    rule_on(&mut s.server, 1, asked.id, true, None);
    assert_eq!(
        block_updates_and_acks(&alice.link.take()),
        [
            "update 2 63 0 block",
            "update 2 62 0 block",
            "update 2 64 0 air",
            "update 2 63 -1 block",
            "update 2 63 1 block",
            "update 1 63 0 block",
            "update 3 63 0 block",
            "ack 2",
            "ack 2",
        ]
    );
    assert_eq!(kind_at(&mut s, DIRT), "minecraft:dirt");
    assert_eq!(s.server.items_on_ground(), 0);
    let settled = settled_for(&s.plugin, EventKind::BlockBreak);
    assert!(settled[0].cancelled);
}

#[test]
fn an_accepted_break_updates_the_block_before_its_ack() {
    let (mut s, alice) = dirt_scene(vec![deciding(1, EventKind::BlockBreak)]);
    mine(&mut s, &alice, DIRT);
    alice.link.take();
    let asked = last_asked(&s.plugin, EventKind::BlockBreak);
    rule_on(&mut s.server, 1, asked.id, false, None);
    let order = block_updates_and_acks(&alice.link.take());
    assert_eq!(order, ["update 2 63 0 air", "ack 2"]);
    assert_eq!(s.server.items_on_ground(), 1);
}

#[test]
fn a_block_changed_while_its_break_waits_is_refused() {
    let (mut s, alice) = dirt_scene(vec![deciding(1, EventKind::BlockBreak)]);
    mine(&mut s, &alice, DIRT);
    let asked = last_asked(&s.plugin, EventKind::BlockBreak);
    set(&mut s, DIRT, "minecraft:gravel");
    rule_on(&mut s.server, 1, asked.id, false, None);
    assert_eq!(kind_at(&mut s, DIRT), "minecraft:gravel");
    assert_eq!(s.server.items_on_ground(), 0);
}

#[test]
fn keeping_the_drops_spawns_nothing_but_still_wears_the_tool() {
    let (mut s, alice) = dirt_scene(vec![
        deciding(1, EventKind::BlockBreak),
        observe(2, EventKind::BlockDropItem),
        observe(3, EventKind::PlayerItemDamage),
    ]);
    console(&mut s, "give Alice iron_shovel");
    mine(&mut s, &alice, DIRT);
    let asked = last_asked(&s.plugin, EventKind::BlockBreak);
    rule_on(
        &mut s.server,
        1,
        asked.id,
        false,
        Some(EventPatch::BlockBreak(BlockBreakPatch {
            drop_items: Some(false),
            exp_to_drop: None,
        })),
    );
    assert_eq!(kind_at(&mut s, DIRT), "minecraft:air");
    assert_eq!(s.server.items_on_ground(), 0);
    assert!(settled_for(&s.plugin, EventKind::BlockDropItem).is_empty());
    assert_eq!(settled_for(&s.plugin, EventKind::PlayerItemDamage).len(), 1);
}

#[test]
fn experience_set_on_a_break_is_paid_even_in_creative() {
    let (mut s, alice) = dirt_scene(vec![deciding(1, EventKind::BlockBreak)]);
    console(&mut s, "gamemode creative Alice");
    dig(&mut s, &alice, DigAction::Start, DIRT, 1);
    let asked = last_asked(&s.plugin, EventKind::BlockBreak);
    rule_on(
        &mut s.server,
        1,
        asked.id,
        false,
        Some(EventPatch::BlockBreak(BlockBreakPatch {
            drop_items: None,
            exp_to_drop: Some(7),
        })),
    );
    assert_eq!(kind_at(&mut s, DIRT), "minecraft:air");
    let paid: i32 = s
        .server
        .experience_orbs()
        .iter()
        .map(|orb| orb.value * orb.count)
        .sum();
    assert_eq!(paid, 7);
    assert_eq!(s.server.items_on_ground(), 0);
}

#[test]
fn a_sword_in_creative_breaks_nothing_and_is_heard_cancelled_without_asking() {
    let (mut s, alice) = dirt_scene(vec![
        deciding(1, EventKind::BlockBreak),
        observe(2, EventKind::BlockBreak),
    ]);
    console(&mut s, "gamemode creative Alice");
    console(&mut s, "give Alice diamond_sword");
    alice.link.take();
    dig(&mut s, &alice, DigAction::Start, DIRT, 4);
    assert!(asked_for(&s.plugin, EventKind::BlockBreak).is_empty());
    let heard = settled_for(&s.plugin, EventKind::BlockBreak);
    assert_eq!(heard.len(), 1);
    assert!(heard[0].cancelled);
    assert_eq!(kind_at(&mut s, DIRT), "minecraft:dirt");
    assert_eq!(block_updates_and_acks(&alice.link.take()), ["ack 4"]);
}

#[test]
fn a_cancelled_damage_resyncs_the_block_and_leaves_no_session() {
    let (mut s, alice) = dirt_scene(vec![deciding(1, EventKind::BlockDamage)]);
    dig(&mut s, &alice, DigAction::Start, DIRT, 1);
    assert!(block_updates_and_acks(&alice.link.take()).is_empty());
    let asked = last_asked(&s.plugin, EventKind::BlockDamage);
    rule_on(&mut s.server, 1, asked.id, true, None);
    assert_eq!(
        block_updates_and_acks(&alice.link.take()),
        ["update 2 63 0 block", "ack 1"]
    );
    dig(&mut s, &alice, DigAction::Finish, DIRT, 2);
    assert_eq!(kind_at(&mut s, DIRT), "minecraft:dirt");
    assert_eq!(s.server.items_on_ground(), 0);
}

#[test]
fn a_finish_sent_while_the_damage_waits_is_judged_once_it_settles() {
    let (mut s, alice) = dirt_scene(vec![deciding(1, EventKind::BlockDamage)]);
    dig(&mut s, &alice, DigAction::Start, DIRT, 1);
    dig(&mut s, &alice, DigAction::Finish, DIRT, 2);
    assert_eq!(kind_at(&mut s, DIRT), "minecraft:dirt");
    let asked = last_asked(&s.plugin, EventKind::BlockDamage);
    rule_on(&mut s.server, 1, asked.id, false, None);
    assert_eq!(kind_at(&mut s, DIRT), "minecraft:dirt");
    assert_eq!(
        block_updates_and_acks(&alice.link.take()),
        ["ack 1", "ack 2"]
    );
    s.tick(15);
    assert_eq!(kind_at(&mut s, DIRT), "minecraft:air");
    assert_eq!(
        block_updates_and_acks(&alice.link.take()),
        ["update 2 63 0 air"]
    );
}

#[test]
fn insta_break_set_on_the_damage_breaks_at_once_and_clearing_it_does_nothing() {
    let (mut s, alice) = dirt_scene(vec![deciding(1, EventKind::BlockDamage)]);
    dig(&mut s, &alice, DigAction::Start, DIRT, 1);
    let asked = last_asked(&s.plugin, EventKind::BlockDamage);
    rule_on(
        &mut s.server,
        1,
        asked.id,
        false,
        Some(EventPatch::BlockDamage(BlockDamagePatch {
            insta_break: Some(true),
        })),
    );
    assert_eq!(kind_at(&mut s, DIRT), "minecraft:air");
    let flower = BlockPos::new(3, 64, 0);
    set(&mut s, flower, "minecraft:dandelion");
    dig(&mut s, &alice, DigAction::Start, flower, 2);
    let asked = last_asked(&s.plugin, EventKind::BlockDamage);
    match &asked.payload {
        EventPayload::BlockDamage(damage) => assert!(damage.insta_break),
        other => panic!("expected a damage, got {other:?}"),
    }
    rule_on(
        &mut s.server,
        1,
        asked.id,
        false,
        Some(EventPatch::BlockDamage(BlockDamagePatch {
            insta_break: Some(false),
        })),
    );
    assert_eq!(kind_at(&mut s, flower), "minecraft:air");
}

#[test]
fn letting_go_fires_the_abort_and_ends_the_session() {
    let (mut s, alice) = dirt_scene(vec![observe(1, EventKind::BlockDamageAbort)]);
    dig(&mut s, &alice, DigAction::Start, DIRT, 1);
    alice.link.take();
    dig(&mut s, &alice, DigAction::Cancel, DIRT, 2);
    let heard = settled_for(&s.plugin, EventKind::BlockDamageAbort);
    assert_eq!(heard.len(), 1);
    match &heard[0].payload {
        EventPayload::BlockDamageAbort(BlockDamageAbort { block, .. }) => {
            assert_eq!(*block, DIRT);
        }
        other => panic!("expected an abort, got {other:?}"),
    }
    assert_eq!(block_updates_and_acks(&alice.link.take()), ["ack 2"]);
    dig(&mut s, &alice, DigAction::Finish, DIRT, 3);
    assert_eq!(kind_at(&mut s, DIRT), "minecraft:dirt");
    let far = BlockPos::new(40, 63, 0);
    dig(&mut s, &alice, DigAction::Cancel, far, 4);
    assert_eq!(settled_for(&s.plugin, EventKind::BlockDamageAbort).len(), 1);
}

#[test]
fn the_abort_waits_behind_the_damage_of_its_player() {
    let (mut s, alice) = dirt_scene(vec![
        deciding(1, EventKind::BlockDamage),
        observe(2, EventKind::BlockDamageAbort),
    ]);
    dig(&mut s, &alice, DigAction::Start, DIRT, 1);
    dig(&mut s, &alice, DigAction::Cancel, DIRT, 2);
    assert!(settled_for(&s.plugin, EventKind::BlockDamageAbort).is_empty());
    assert!(block_updates_and_acks(&alice.link.take()).is_empty());
    let asked = last_asked(&s.plugin, EventKind::BlockDamage);
    rule_on(&mut s.server, 1, asked.id, false, None);
    let kinds: Vec<(Phase, EventKind)> = fired_kinds(&s.plugin)
        .into_iter()
        .filter(|(phase, _)| *phase == Phase::Settled)
        .collect();
    assert_eq!(kinds, [(Phase::Settled, EventKind::BlockDamageAbort)]);
    assert_eq!(
        block_updates_and_acks(&alice.link.take()),
        ["ack 1", "ack 2"]
    );
}

#[test]
fn a_drop_patch_keeps_only_the_listed_items_and_a_cancel_keeps_the_experience() {
    let (mut s, alice) = dirt_scene(vec![
        deciding(1, EventKind::BlockDropItem),
        deciding(2, EventKind::BlockBreak),
    ]);
    mine(&mut s, &alice, DIRT);
    let asked = last_asked(&s.plugin, EventKind::BlockBreak);
    judge(
        &mut s.server,
        1,
        asked.id,
        2,
        false,
        Some(EventPatch::BlockBreak(BlockBreakPatch {
            drop_items: None,
            exp_to_drop: Some(3),
        })),
    );
    let asked = last_asked(&s.plugin, EventKind::BlockDropItem);
    let offered = drop_of(&asked);
    assert_eq!(offered.items.len(), 1);
    judge(
        &mut s.server,
        1,
        asked.id,
        1,
        false,
        Some(EventPatch::BlockDropItem(BlockDropItemPatch {
            items: replacing(&offered.items, Vec::new()),
        })),
    );
    assert_eq!(s.server.items_on_ground(), 0);
    let paid: i32 = s
        .server
        .experience_orbs()
        .iter()
        .map(|orb| orb.value * orb.count)
        .sum();
    assert_eq!(paid, 3);
    let second = BlockPos::new(3, 63, 0);
    set(&mut s, second, "minecraft:dirt");
    mine(&mut s, &alice, second);
    let asked = last_asked(&s.plugin, EventKind::BlockBreak);
    judge(&mut s.server, 1, asked.id, 2, false, None);
    let asked = last_asked(&s.plugin, EventKind::BlockDropItem);
    judge(
        &mut s.server,
        1,
        asked.id,
        1,
        false,
        Some(EventPatch::BlockDropItem(BlockDropItemPatch::default())),
    );
    assert_eq!(s.server.items_on_ground(), 1);
    let third = BlockPos::new(4, 63, 0);
    set(&mut s, third, "minecraft:dirt");
    mine(&mut s, &alice, third);
    let asked = last_asked(&s.plugin, EventKind::BlockBreak);
    judge(&mut s.server, 1, asked.id, 2, false, None);
    let asked = last_asked(&s.plugin, EventKind::BlockDropItem);
    judge(&mut s.server, 1, asked.id, 1, true, None);
    assert_eq!(s.server.items_on_ground(), 1);
    assert_eq!(kind_at(&mut s, third), "minecraft:air");
}

#[test]
fn a_block_without_drops_still_fires_an_empty_drop_event() {
    let (mut s, alice) = dirt_scene(vec![observe(1, EventKind::BlockDropItem)]);
    let glass = BlockPos::new(3, 63, 0);
    set(&mut s, glass, "minecraft:glass");
    mine(&mut s, &alice, glass);
    let dropped = drop_of(&settled_for(&s.plugin, EventKind::BlockDropItem)[0]);
    assert!(dropped.items.is_empty());
    assert_eq!(s.server.items_on_ground(), 0);
}

#[test]
fn a_drop_patch_may_replace_the_stack_of_a_kept_item() {
    let (mut s, alice) = dirt_scene(vec![deciding(1, EventKind::BlockDropItem)]);
    mine(&mut s, &alice, DIRT);
    let asked = last_asked(&s.plugin, EventKind::BlockDropItem);
    let offered = drop_of(&asked).items;
    assert_eq!(offered.len(), 1);
    let mut kept = offered.clone();
    kept[0].item = ItemSpec::of("minecraft:diamond", 3);
    rule_on(
        &mut s.server,
        1,
        asked.id,
        false,
        Some(EventPatch::BlockDropItem(BlockDropItemPatch {
            items: replacing(&offered, kept),
        })),
    );
    let thrown: Vec<String> = s
        .server
        .ground_items()
        .iter()
        .filter_map(|(_, stack)| Some(format!("{} {}", stack.item.kind()?.name(), stack.count)))
        .collect();
    assert_eq!(thrown, ["minecraft:diamond 3"]);
}

#[test]
fn a_cancelled_break_of_a_campfire_resends_its_block_entity_between_two_acks() {
    let (mut s, alice) = dirt_scene(vec![deciding(1, EventKind::BlockBreak)]);
    set(&mut s, DIRT, "minecraft:campfire");
    console(&mut s, "gamemode creative Alice");
    alice.link.take();
    dig(&mut s, &alice, DigAction::Start, DIRT, 6);
    let asked = last_asked(&s.plugin, EventKind::BlockBreak);
    alice.link.take();
    rule_on(&mut s.server, 1, asked.id, true, None);
    let order: Vec<String> = alice
        .link
        .take()
        .iter()
        .filter_map(|event| match event {
            ClientEvent::BlockUpdate { pos, .. } => {
                Some(format!("update {} {} {}", pos.x, pos.y, pos.z))
            }
            ClientEvent::BlockChangedAck(sequence) => Some(format!("ack {sequence}")),
            ClientEvent::BlockEntityUpdate { pos, .. } => {
                Some(format!("entity {} {} {}", pos.x, pos.y, pos.z))
            }
            _ => None,
        })
        .collect();
    assert_eq!(order.len(), 10, "{order:?}");
    assert_eq!(order[0], "update 2 63 0");
    assert_eq!(order[7..], ["ack 6", "entity 2 63 0", "ack 6"]);
    assert_eq!(kind_at(&mut s, DIRT), "minecraft:campfire");
}

const STONE: BlockPos = BlockPos::new(3, 63, 0);

fn pickaxe_scene() -> (Scene, Player) {
    let (mut s, alice) = dirt_scene(vec![observe(1, EventKind::BlockBreak)]);
    console(&mut s, "give Alice iron_pickaxe");
    alice.link.take();
    (s, alice)
}

#[test]
fn a_finish_below_the_progress_threshold_breaks_the_block_once_the_progress_is_due() {
    let (mut s, alice) = pickaxe_scene();
    dig(&mut s, &alice, DigAction::Start, STONE, 1);
    dig(&mut s, &alice, DigAction::Finish, STONE, 2);
    assert_eq!(kind_at(&mut s, STONE), "minecraft:stone");
    assert_eq!(
        block_updates_and_acks(&alice.link.take()),
        ["ack 1", "ack 2"]
    );
    s.tick(5);
    assert_eq!(kind_at(&mut s, STONE), "minecraft:stone");
    s.tick(5);
    assert_eq!(kind_at(&mut s, STONE), "minecraft:air");
    assert_eq!(settled_for(&s.plugin, EventKind::BlockBreak).len(), 1);
    assert_eq!(
        block_updates_and_acks(&alice.link.take()),
        ["update 3 63 0 air"]
    );
}

#[test]
fn a_finish_at_the_progress_threshold_breaks_the_block_at_once() {
    let (mut s, alice) = pickaxe_scene();
    dig(&mut s, &alice, DigAction::Start, STONE, 1);
    s.tick(5);
    dig(&mut s, &alice, DigAction::Finish, STONE, 2);
    assert_eq!(kind_at(&mut s, STONE), "minecraft:air");
    assert_eq!(settled_for(&s.plugin, EventKind::BlockBreak).len(), 1);
}

#[test]
fn an_insta_break_still_breaks_at_the_start() {
    let (mut s, alice) = pickaxe_scene();
    let flower = BlockPos::new(3, 64, 0);
    set(&mut s, flower, "minecraft:poppy");
    dig(&mut s, &alice, DigAction::Start, flower, 1);
    assert_eq!(kind_at(&mut s, flower), "minecraft:air");
}

#[test]
fn an_abort_does_not_call_off_a_delayed_destroy_as_in_the_bytecode() {
    let (mut s, alice) = pickaxe_scene();
    dig(&mut s, &alice, DigAction::Start, STONE, 1);
    dig(&mut s, &alice, DigAction::Finish, STONE, 2);
    dig(&mut s, &alice, DigAction::Cancel, STONE, 3);
    dig(&mut s, &alice, DigAction::Start, DIRT, 4);
    s.tick(10);
    assert_eq!(kind_at(&mut s, STONE), "minecraft:air");
}

#[test]
fn a_born_cancelled_break_nobody_listens_to_applies_at_once_behind_a_waiting_move() {
    let (mut s, alice) = dirt_scene(vec![decide(1)]);
    console(&mut s, "gamemode creative Alice");
    console(&mut s, "give Alice diamond_sword");
    send_input(
        &mut s.server,
        &alice,
        PlayerInput::Move {
            position: Some(rspigot_domain::Vec3::new(0.6, 64.0, 0.5)),
            rotation: None,
            on_ground: true,
        },
    );
    assert_eq!(asked_for(&s.plugin, EventKind::PlayerMove).len(), 1);
    alice.link.take();
    dig(&mut s, &alice, DigAction::Start, DIRT, 4);
    assert_eq!(block_updates_and_acks(&alice.link.take()), ["ack 4"]);
    assert_eq!(kind_at(&mut s, DIRT), "minecraft:dirt");
}
