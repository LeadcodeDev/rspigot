mod common;

use common::*;
use rspigot_core::events::{ClientEvent, DigAction, Direction, Hand, Inbound, PlayerInput};
use rspigot_core::plugins::events::{
    Action, EventResult, Face, PlayerDropItem, PlayerInteract, PlayerInteractAtEntity,
    PlayerInteractPatch, PlayerItemBreak, PlayerItemDamage, PlayerItemDamagePatch,
    PlayerTakeLecternBook,
};
use rspigot_core::plugins::{
    Answer, EventKind, EventPatch, EventPayload, ItemSpec, Phase, Question,
};
use rspigot_domain::{BlockPos, EntityId, Rotation, Vec3};
use rspigot_entities::EntityKind as Kind;

const FLOOR: BlockPos = BlockPos::new(2, 63, 0);
const ABOVE: BlockPos = BlockPos::new(2, 64, 0);

fn console(s: &mut Scene, line: &str) {
    s.server.handle(Inbound::Console(line.to_owned()));
    s.server.deliver();
}

fn use_on(s: &mut Scene, player: &Player, pos: BlockPos, sequence: i32) {
    send_input(
        &mut s.server,
        player,
        PlayerInput::UseItemOn {
            hand: Hand::Main,
            pos,
            face: Direction::Up,
            cursor: Vec3::new(0.5, 1.0, 0.5),
            inside_block: false,
            sequence,
        },
    );
}

fn dig(s: &mut Scene, player: &Player, action: DigAction, pos: BlockPos) {
    send_input(
        &mut s.server,
        player,
        PlayerInput::Dig {
            action,
            pos,
            face: Direction::Up,
            sequence: 3,
        },
    );
}

fn block_is_air(s: &mut Scene, pos: BlockPos) -> bool {
    match s.ask(Question::Block(pos)) {
        Answer::Block(state) => state.kind == "minecraft:air",
        other => panic!("expected a block, got {other:?}"),
    }
}

fn inventory(s: &mut Scene, player: &Player) -> Vec<Option<ItemSpec>> {
    match s.ask(Question::Inventory(player.id)) {
        Answer::Inventory(inventory) => inventory.slots,
        other => panic!("expected the inventory, got {other:?}"),
    }
}

fn interact_of(fired: &Fired) -> PlayerInteract {
    match &fired.payload {
        EventPayload::PlayerInteract(interact) => interact.clone(),
        other => panic!("expected an interact, got {other:?}"),
    }
}

fn results(block: EventResult, item: EventResult) -> Option<EventPatch> {
    Some(EventPatch::PlayerInteract(PlayerInteractPatch {
        use_interacted_block: Some(block),
        use_item_in_hand: Some(item),
    }))
}

#[test]
fn a_right_click_on_a_block_describes_the_click_and_its_item() {
    let mut s = scene_with(&["Alice"], vec![observe(1, EventKind::PlayerInteract)]);
    let alice = s.players[0].clone();
    console(&mut s, "give Alice stone 4");
    use_on(&mut s, &alice, FLOOR, 1);
    let heard = settled_for(&s.plugin, EventKind::PlayerInteract);
    let interact = interact_of(&heard[0]);
    assert_eq!(interact.action, Action::RightClickBlock);
    assert!(interact.block);
    assert_eq!(interact.clicked_block, Some(FLOOR));
    assert_eq!(interact.block_face, Face::Up);
    assert_eq!(interact.clicked_position, Some(Vec3::new(0.5, 1.0, 0.5)));
    assert_eq!(interact.item, Some(ItemSpec::of("minecraft:stone", 4)));
    assert!(interact.block_in_hand && interact.has_item);
    assert_eq!(interact.material, "minecraft:stone");
    assert_eq!(interact.use_interacted_block, EventResult::Allow);
    assert_eq!(interact.use_item_in_hand, EventResult::Default);
    assert!(!heard[0].cancelled);
    assert!(!block_is_air(&mut s, ABOVE));
}

#[test]
fn denying_the_item_in_hand_keeps_the_block_from_being_placed() {
    let mut s = scene_with(&["Alice"], vec![deciding(1, EventKind::PlayerInteract)]);
    let alice = s.players[0].clone();
    console(&mut s, "give Alice stone 4");
    use_on(&mut s, &alice, FLOOR, 1);
    assert!(block_is_air(&mut s, ABOVE));
    let asked = last_asked(&s.plugin, EventKind::PlayerInteract);
    rule_on(
        &mut s.server,
        1,
        asked.id,
        false,
        results(EventResult::Allow, EventResult::Deny),
    );
    assert!(block_is_air(&mut s, ABOVE));
    assert_eq!(
        inventory(&mut s, &alice)[0],
        Some(ItemSpec::of("minecraft:stone", 4))
    );
    use_on(&mut s, &alice, FLOOR, 2);
    let asked = last_asked(&s.plugin, EventKind::PlayerInteract);
    rule_on(&mut s.server, 1, asked.id, true, None);
    assert!(block_is_air(&mut s, ABOVE));
    use_on(&mut s, &alice, FLOOR, 3);
    let asked = last_asked(&s.plugin, EventKind::PlayerInteract);
    rule_on(&mut s.server, 1, asked.id, false, None);
    assert!(!block_is_air(&mut s, ABOVE));
}

#[test]
fn a_click_whose_block_changed_while_parked_is_refused() {
    let mut s = scene_with(&["Alice"], vec![deciding(1, EventKind::PlayerInteract)]);
    let alice = s.players[0].clone();
    console(&mut s, "give Alice stone 4");
    use_on(&mut s, &alice, FLOOR, 1);
    let asked = last_asked(&s.plugin, EventKind::PlayerInteract);
    console(&mut s, "clear Alice");
    rule_on(&mut s.server, 1, asked.id, false, None);
    assert!(block_is_air(&mut s, ABOVE));
}

#[test]
fn a_right_click_in_the_air_starts_denied_and_its_item_result_still_counts() {
    let mut s = scene_with(&["Alice"], vec![deciding(1, EventKind::PlayerInteract)]);
    let alice = s.players[0].clone();
    console(&mut s, "give Alice bow 1");
    alice.link.take();
    send_input(
        &mut s.server,
        &alice,
        PlayerInput::UseItem {
            hand: Hand::Main,
            sequence: 4,
            rotation: Rotation::new(0.0, -90.0),
        },
    );
    let asked = last_asked(&s.plugin, EventKind::PlayerInteract);
    let interact = interact_of(&asked);
    assert_eq!(interact.action, Action::RightClickAir);
    assert!(!interact.block);
    assert_eq!(interact.use_interacted_block, EventResult::Deny);
    rule_on(
        &mut s.server,
        1,
        asked.id,
        false,
        results(EventResult::Deny, EventResult::Deny),
    );
    let events = alice.link.take();
    assert!(
        events
            .iter()
            .any(|event| matches!(event, ClientEvent::InventoryContent { .. }))
    );
    assert!(
        events
            .iter()
            .any(|event| matches!(event, ClientEvent::BlockChangedAck(4)))
    );
    let heard = settled_for(&s.plugin, EventKind::PlayerInteract);
    assert!(heard.is_empty());
}

#[test]
fn an_empty_hand_in_the_air_fires_nothing() {
    let mut s = scene_with(&["Alice"], vec![observe(1, EventKind::PlayerInteract)]);
    let alice = s.players[0].clone();
    send_input(
        &mut s.server,
        &alice,
        PlayerInput::UseItem {
            hand: Hand::Main,
            sequence: 4,
            rotation: Rotation::new(0.0, -90.0),
        },
    );
    assert!(settled_for(&s.plugin, EventKind::PlayerInteract).is_empty());
}

#[test]
fn a_cancelled_left_click_keeps_the_block_and_refuses_its_finish() {
    let mut s = scene_with(&["Alice"], vec![deciding(1, EventKind::PlayerInteract)]);
    let alice = s.players[0].clone();
    dig(&mut s, &alice, DigAction::Start, FLOOR);
    let asked = last_asked(&s.plugin, EventKind::PlayerInteract);
    let interact = interact_of(&asked);
    assert_eq!(interact.action, Action::LeftClickBlock);
    assert_eq!(interact.clicked_block, Some(FLOOR));
    alice.link.take();
    rule_on(&mut s.server, 1, asked.id, true, None);
    assert!(
        alice
            .link
            .take()
            .iter()
            .any(|event| matches!(event, ClientEvent::BlockUpdate { pos, .. } if *pos == FLOOR))
    );
    dig(&mut s, &alice, DigAction::Finish, FLOOR);
    assert!(!block_is_air(&mut s, FLOOR));
}

#[test]
fn an_allowed_left_click_in_creative_breaks_the_block() {
    let mut s = scene_with(&["Alice"], vec![deciding(1, EventKind::PlayerInteract)]);
    let alice = s.players[0].clone();
    console(&mut s, "gamemode creative Alice");
    dig(&mut s, &alice, DigAction::Start, FLOOR);
    assert!(!block_is_air(&mut s, FLOOR));
    let asked = last_asked(&s.plugin, EventKind::PlayerInteract);
    rule_on(&mut s.server, 1, asked.id, false, None);
    assert!(block_is_air(&mut s, FLOOR));
}

#[test]
fn a_punch_in_the_air_is_only_observed_then_the_swing_is_asked() {
    let mut s = scene_with(
        &["Alice"],
        vec![
            deciding(1, EventKind::PlayerInteract),
            observe(2, EventKind::PlayerInteract),
            observe(3, EventKind::PlayerAnimation),
        ],
    );
    let alice = s.players[0].clone();
    send_input(
        &mut s.server,
        &alice,
        PlayerInput::Move {
            position: None,
            rotation: Some(Rotation::new(0.0, -90.0)),
            on_ground: true,
        },
    );
    send_input(&mut s.server, &alice, PlayerInput::Swing(Hand::Main));
    let kinds = fired_kinds(&s.plugin);
    assert_eq!(
        kinds,
        [
            (Phase::Settled, EventKind::PlayerInteract),
            (Phase::Settled, EventKind::PlayerAnimation),
        ]
    );
    let interact = interact_of(&fired(&s.plugin)[0]);
    assert_eq!(interact.action, Action::LeftClickAir);
    assert!(fired(&s.plugin)[0].cancelled);
}

fn cow_near(s: &mut Scene) -> EntityId {
    let before: Vec<EntityId> = s
        .server
        .entities_of(Kind::Cow)
        .iter()
        .map(|e| e.id)
        .collect();
    console(s, "summon cow 1.5 64 0.5");
    s.server
        .entities_of(Kind::Cow)
        .iter()
        .map(|e| e.id)
        .find(|id| !before.contains(id))
        .expect("a cow")
}

fn interact_with(s: &mut Scene, player: &Player, target: EntityId) {
    send_input(
        &mut s.server,
        player,
        PlayerInput::Interact {
            target,
            hand: Hand::Main,
            sneaking: false,
            location: Vec3::new(0.1, 0.9, -0.2),
        },
    );
}

#[test]
fn a_cancelled_entity_click_does_not_milk_the_cow_and_reaches_both_handler_types() {
    let mut s = scene_with(
        &["Alice"],
        vec![
            deciding(1, EventKind::PlayerInteractAtEntity),
            observe(2, EventKind::PlayerInteractEntity),
        ],
    );
    let alice = s.players[0].clone();
    console(&mut s, "give Alice bucket 1");
    let cow = cow_near(&mut s);
    interact_with(&mut s, &alice, cow);
    let asked = last_asked(&s.plugin, EventKind::PlayerInteractAtEntity);
    let EventPayload::PlayerInteractAtEntity(PlayerInteractAtEntity {
        clicked_position,
        right_clicked,
        ..
    }) = &asked.payload
    else {
        panic!("expected an entity click");
    };
    assert_eq!(*clicked_position, Vec3::new(0.1, 0.9, -0.2));
    assert_eq!(right_clicked.id, cow);
    rule_on(&mut s.server, 1, asked.id, true, None);
    assert_eq!(
        inventory(&mut s, &alice)[0],
        Some(ItemSpec::of("minecraft:bucket", 1))
    );
    let heard = settled_for(&s.plugin, EventKind::PlayerInteractAtEntity);
    assert_eq!(heard.len(), 1);
    assert!(heard[0].cancelled);
    interact_with(&mut s, &alice, cow);
    let asked = last_asked(&s.plugin, EventKind::PlayerInteractAtEntity);
    rule_on(&mut s.server, 1, asked.id, false, None);
    assert_eq!(
        inventory(&mut s, &alice)[0],
        Some(ItemSpec::of("minecraft:milk_bucket", 1))
    );
}

fn drop_one(s: &mut Scene, player: &Player) {
    dig(s, player, DigAction::DropItem, BlockPos::new(0, 0, 0));
}

#[test]
fn a_cancelled_drop_keeps_the_item_and_an_allowed_one_spawns_the_announced_entity() {
    let mut s = scene_with(&["Alice"], vec![deciding(1, EventKind::PlayerDropItem)]);
    let alice = s.players[0].clone();
    console(&mut s, "give Alice diamond 3");
    drop_one(&mut s, &alice);
    let asked = last_asked(&s.plugin, EventKind::PlayerDropItem);
    let EventPayload::PlayerDropItem(PlayerDropItem { item_drop, .. }) = &asked.payload else {
        panic!("expected a drop");
    };
    assert_eq!(item_drop.kind, "minecraft:item");
    assert_eq!(s.server.items_on_ground(), 0);
    rule_on(&mut s.server, 1, asked.id, true, None);
    assert_eq!(s.server.items_on_ground(), 0);
    assert_eq!(
        inventory(&mut s, &alice)[0],
        Some(ItemSpec::of("minecraft:diamond", 3))
    );
    drop_one(&mut s, &alice);
    let asked = last_asked(&s.plugin, EventKind::PlayerDropItem);
    let EventPayload::PlayerDropItem(PlayerDropItem { item_drop, .. }) = &asked.payload else {
        panic!("expected a drop");
    };
    let announced = item_drop.id;
    rule_on(&mut s.server, 1, asked.id, false, None);
    assert_eq!(s.server.items_on_ground(), 1);
    assert!(
        s.server
            .entities_of(Kind::Item)
            .iter()
            .any(|item| item.id == announced)
    );
    assert_eq!(
        inventory(&mut s, &alice)[0],
        Some(ItemSpec::of("minecraft:diamond", 2))
    );
}

fn mine_with_a_pickaxe(s: &mut Scene, player: &Player) {
    dig(s, player, DigAction::Finish, BlockPos::new(3, 63, 3));
}

fn damage_of(s: &mut Scene, player: &Player) -> Vec<u8> {
    match &inventory(s, player)[0] {
        Some(spec) => spec.components.clone(),
        None => panic!("the pickaxe is gone"),
    }
}

#[test]
fn a_tool_wear_is_asked_its_damage_patched_and_a_break_follows_it() {
    let mut s = scene_with(
        &["Alice"],
        vec![
            deciding(1, EventKind::PlayerItemDamage),
            observe(2, EventKind::PlayerItemBreak),
        ],
    );
    let alice = s.players[0].clone();
    console(&mut s, "give Alice stone_pickaxe 1");
    let fresh = damage_of(&mut s, &alice);
    mine_with_a_pickaxe(&mut s, &alice);
    let asked = last_asked(&s.plugin, EventKind::PlayerItemDamage);
    let EventPayload::PlayerItemDamage(PlayerItemDamage { damage, item, .. }) = &asked.payload
    else {
        panic!("expected a wear");
    };
    assert_eq!(*damage, 1);
    assert_eq!(item.key, "minecraft:stone_pickaxe");
    rule_on(&mut s.server, 1, asked.id, true, None);
    assert_eq!(damage_of(&mut s, &alice), fresh);
    dig(&mut s, &alice, DigAction::Finish, BlockPos::new(4, 63, 3));
    let asked = last_asked(&s.plugin, EventKind::PlayerItemDamage);
    rule_on(
        &mut s.server,
        1,
        asked.id,
        false,
        Some(EventPatch::PlayerItemDamage(PlayerItemDamagePatch {
            damage: Some(10_000),
        })),
    );
    assert_eq!(inventory(&mut s, &alice)[0], None);
    let broken = settled_for(&s.plugin, EventKind::PlayerItemBreak);
    let EventPayload::PlayerItemBreak(PlayerItemBreak { broken_item, .. }) = &broken[0].payload
    else {
        panic!("expected a break");
    };
    assert_eq!(broken_item.key, "minecraft:stone_pickaxe");
}

#[test]
fn a_tool_wear_without_a_decider_is_not_asked_and_one_in_flight_blocks_the_next() {
    let mut s = scene_with(&["Alice"], vec![observe(1, EventKind::PlayerItemDamage)]);
    let alice = s.players[0].clone();
    console(&mut s, "give Alice stone_pickaxe 1");
    mine_with_a_pickaxe(&mut s, &alice);
    assert!(asked_for(&s.plugin, EventKind::PlayerItemDamage).is_empty());
    assert_eq!(settled_for(&s.plugin, EventKind::PlayerItemDamage).len(), 1);

    let mut s = scene_with(&["Alice"], vec![deciding(1, EventKind::PlayerItemDamage)]);
    let alice = s.players[0].clone();
    console(&mut s, "give Alice stone_pickaxe 1");
    mine_with_a_pickaxe(&mut s, &alice);
    dig(&mut s, &alice, DigAction::Finish, BlockPos::new(4, 63, 3));
    assert_eq!(asked_for(&s.plugin, EventKind::PlayerItemDamage).len(), 1);
}

#[test]
fn a_creative_player_wears_nothing_but_the_event_still_fires_with_zero() {
    let mut s = scene_with(&["Alice"], vec![deciding(1, EventKind::PlayerItemDamage)]);
    let alice = s.players[0].clone();
    console(&mut s, "give Alice stone_sword 1");
    console(&mut s, "gamemode creative Alice");
    let fresh = damage_of(&mut s, &alice);
    let cow = cow_near(&mut s);
    send_input(&mut s.server, &alice, PlayerInput::Attack(cow));
    let asked = last_asked(&s.plugin, EventKind::PlayerItemDamage);
    let EventPayload::PlayerItemDamage(PlayerItemDamage { damage, .. }) = &asked.payload else {
        panic!("expected a wear");
    };
    assert_eq!(*damage, 0);
    rule_on(&mut s.server, 1, asked.id, false, None);
    assert_eq!(damage_of(&mut s, &alice), fresh);
}

fn window_opened(player: &Player) -> Option<i32> {
    player
        .link
        .take()
        .into_iter()
        .find_map(|event| match event {
            ClientEvent::OpenScreen { window, .. } => Some(window),
            _ => None,
        })
}

fn lectern_with_a_book(s: &mut Scene, player: &Player) -> i32 {
    console(s, "give Alice lectern 1");
    use_on(s, player, FLOOR, 1);
    console(s, "give Alice writable_book 1");
    use_on(s, player, ABOVE, 2);
    player.link.take();
    use_on(s, player, ABOVE, 3);
    window_opened(player).expect("the lectern opened")
}

#[test]
fn a_cancelled_book_take_leaves_the_book_on_the_lectern() {
    let mut s = scene_with(
        &["Alice"],
        vec![deciding(1, EventKind::PlayerTakeLecternBook)],
    );
    let alice = s.players[0].clone();
    let window = lectern_with_a_book(&mut s, &alice);
    send_input(
        &mut s.server,
        &alice,
        PlayerInput::ContainerButtonClick { window, button: 3 },
    );
    let asked = last_asked(&s.plugin, EventKind::PlayerTakeLecternBook);
    let EventPayload::PlayerTakeLecternBook(PlayerTakeLecternBook { book, lectern, .. }) =
        &asked.payload
    else {
        panic!("expected a book take");
    };
    assert_eq!(
        book.as_ref().map(|b| b.key.as_str()),
        Some("minecraft:writable_book")
    );
    assert_eq!(lectern.pos, ABOVE);
    rule_on(&mut s.server, 1, asked.id, true, None);
    assert!(
        !inventory(&mut s, &alice)
            .iter()
            .flatten()
            .any(|item| item.key == "minecraft:writable_book")
    );
    send_input(
        &mut s.server,
        &alice,
        PlayerInput::ContainerButtonClick { window, button: 3 },
    );
    let asked = last_asked(&s.plugin, EventKind::PlayerTakeLecternBook);
    rule_on(&mut s.server, 1, asked.id, false, None);
    assert!(
        inventory(&mut s, &alice)
            .iter()
            .flatten()
            .any(|item| item.key == "minecraft:writable_book")
    );
}
