mod common;

use common::*;
use rspigot_core::events::{ClientEvent, DigAction, Direction, Hand, Inbound, PlayerInput};
use rspigot_core::plugins::events::{
    Action, EventResult, Face, PlayerDropItem, PlayerDropItemPatch, PlayerInteract,
    PlayerInteractAtEntity, PlayerInteractPatch, PlayerItemBreak, PlayerItemDamage,
    PlayerItemDamagePatch, PlayerTakeLecternBook,
};
use rspigot_core::plugins::{
    Answer, EventKind, EventPatch, EventPayload, ItemSpec, Phase, PlayerOrder, Question,
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
fn a_punch_in_the_air_asks_its_deciders_without_waiting_and_drops_their_verdicts() {
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
            (Phase::Decide, EventKind::PlayerInteract),
            (Phase::Settled, EventKind::PlayerInteract),
            (Phase::Settled, EventKind::PlayerAnimation),
        ]
    );
    let asked = last_asked(&s.plugin, EventKind::PlayerInteract);
    assert_eq!(interact_of(&asked).action, Action::LeftClickAir);
    assert!(fired(&s.plugin)[1].cancelled);
    rule_on(
        &mut s.server,
        1,
        asked.id,
        true,
        results(EventResult::Allow, EventResult::Allow),
    );
    assert!(s.warnings().is_empty());
    assert_eq!(fired(&s.plugin).len(), 3);
}

#[test]
fn a_left_click_on_the_protected_spawn_asks_its_deciders_and_parks_nothing() {
    let mut server = configured(|config| config.spawn_protection = 16);
    let plugin = connect_with(
        &mut server,
        1,
        "tester",
        vec![deciding(1, EventKind::PlayerInteract)],
    );
    let alice = join(&mut server, "Alice");
    let bob = join(&mut server, "Bob");
    let mut s = Scene {
        server,
        plugin,
        players: vec![alice.clone(), bob],
    };
    dig(&mut s, &alice, DigAction::Start, FLOOR);
    let asked = last_asked(&s.plugin, EventKind::PlayerInteract);
    assert_eq!(interact_of(&asked).action, Action::LeftClickBlock);
    dig(&mut s, &alice, DigAction::Finish, FLOOR);
    assert!(!block_is_air(&mut s, FLOOR));
    rule_on(&mut s.server, 1, asked.id, false, None);
    assert!(s.warnings().is_empty());
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
fn a_cancelled_drop_keeps_the_item_and_an_allowed_one_spawns_its_stack() {
    let mut s = scene_with(&["Alice"], vec![deciding(1, EventKind::PlayerDropItem)]);
    let alice = s.players[0].clone();
    console(&mut s, "give Alice diamond 3");
    drop_one(&mut s, &alice);
    let asked = last_asked(&s.plugin, EventKind::PlayerDropItem);
    let EventPayload::PlayerDropItem(PlayerDropItem { item, .. }) = &asked.payload else {
        panic!("expected a drop");
    };
    assert_eq!(*item, ItemSpec::of("minecraft:diamond", 1));
    assert_eq!(s.server.items_on_ground(), 0);
    rule_on(&mut s.server, 1, asked.id, true, None);
    assert_eq!(s.server.items_on_ground(), 0);
    assert_eq!(
        inventory(&mut s, &alice)[0],
        Some(ItemSpec::of("minecraft:diamond", 3))
    );
    drop_one(&mut s, &alice);
    let asked = last_asked(&s.plugin, EventKind::PlayerDropItem);
    rule_on(&mut s.server, 1, asked.id, false, None);
    assert_eq!(s.server.items_on_ground(), 1);
    assert_eq!(
        inventory(&mut s, &alice)[0],
        Some(ItemSpec::of("minecraft:diamond", 2))
    );
}

#[test]
fn a_changed_drop_stack_is_the_one_thrown() {
    let mut s = scene_with(&["Alice"], vec![deciding(1, EventKind::PlayerDropItem)]);
    let alice = s.players[0].clone();
    console(&mut s, "give Alice diamond 3");
    let entities_before = s.server.entities_of(Kind::Item).len();
    drop_one(&mut s, &alice);
    assert_eq!(s.server.entities_of(Kind::Item).len(), entities_before);
    let asked = last_asked(&s.plugin, EventKind::PlayerDropItem);
    rule_on(
        &mut s.server,
        1,
        asked.id,
        false,
        Some(EventPatch::PlayerDropItem(PlayerDropItemPatch {
            item: Some(ItemSpec::of("minecraft:gold_ingot", 2)),
        })),
    );
    let thrown: Vec<String> = s
        .server
        .ground_items()
        .iter()
        .filter_map(|(_, stack)| Some(format!("{} {}", stack.item.kind()?.name(), stack.count)))
        .collect();
    assert_eq!(thrown, ["minecraft:gold_ingot 2"]);
    assert_eq!(
        inventory(&mut s, &alice)[0],
        Some(ItemSpec::of("minecraft:diamond", 2))
    );
}

fn mine_with_a_pickaxe(s: &mut Scene, player: &Player) {
    mine_at(s, player, BlockPos::new(3, 63, 3));
}

fn mine_at(s: &mut Scene, player: &Player, pos: BlockPos) {
    send_input(
        &mut s.server,
        player,
        PlayerInput::Move {
            position: None,
            rotation: None,
            on_ground: true,
        },
    );
    dig(s, player, DigAction::Start, pos);
    s.tick(10);
    dig(s, player, DigAction::Finish, pos);
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
    mine_at(&mut s, &alice, BlockPos::new(4, 63, 3));
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
fn a_tool_wear_without_a_decider_is_not_asked_and_every_wear_queues() {
    let mut s = scene_with(&["Alice"], vec![observe(1, EventKind::PlayerItemDamage)]);
    let alice = s.players[0].clone();
    console(&mut s, "give Alice stone_pickaxe 1");
    mine_with_a_pickaxe(&mut s, &alice);
    assert!(asked_for(&s.plugin, EventKind::PlayerItemDamage).is_empty());
    assert_eq!(settled_for(&s.plugin, EventKind::PlayerItemDamage).len(), 1);

    let patient = rspigot_core::config::PluginSettings {
        decision_ticks: 100,
        ..rspigot_core::config::PluginSettings::default()
    };
    let mut s = scene_on(
        server(patient),
        &["Alice"],
        vec![deciding(1, EventKind::PlayerItemDamage)],
    );
    let alice = s.players[0].clone();
    console(&mut s, "give Alice stone_sword 1");
    let fresh = damage_of(&mut s, &alice);
    let cow = cow_near(&mut s);
    send_input(&mut s.server, &alice, PlayerInput::Attack(cow));
    s.tick(11);
    send_input(&mut s.server, &alice, PlayerInput::Attack(cow));
    let asked = asked_for(&s.plugin, EventKind::PlayerItemDamage);
    assert_eq!(asked.len(), 1);
    rule_on(&mut s.server, 1, asked[0].id, false, None);
    let asked = asked_for(&s.plugin, EventKind::PlayerItemDamage);
    assert_eq!(asked.len(), 2);
    let once = damage_of(&mut s, &alice);
    assert_ne!(once, fresh);
    rule_on(&mut s.server, 1, asked[1].id, false, None);
    assert_ne!(damage_of(&mut s, &alice), once);
}

#[test]
fn a_busy_lane_without_a_wear_listener_wears_the_tool_at_once() {
    let mut s = scene_with(&["Alice"], vec![decide(1)]);
    let alice = s.players[0].clone();
    console(&mut s, "give Alice stone_sword 1");
    let mut worn = damage_of(&mut s, &alice);
    let cow = cow_near(&mut s);
    for x in [0.6, 0.7] {
        send_input(
            &mut s.server,
            &alice,
            PlayerInput::Move {
                position: Some(Vec3::new(x, 64.0, 0.5)),
                rotation: None,
                on_ground: true,
            },
        );
        send_input(&mut s.server, &alice, PlayerInput::Attack(cow));
        let now = damage_of(&mut s, &alice);
        assert_ne!(now, worn, "x {x}");
        worn = now;
        s.tick(11);
    }
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

#[test]
fn taking_a_book_into_a_full_inventory_fires_the_take_then_the_drop() {
    let mut s = scene_with(
        &["Alice"],
        vec![
            observe(1, EventKind::PlayerTakeLecternBook),
            observe(2, EventKind::PlayerDropItem),
        ],
    );
    let alice = s.players[0].clone();
    console(&mut s, "give Alice lectern 1");
    use_on(&mut s, &alice, FLOOR, 1);
    console(&mut s, "give Alice writable_book 1");
    use_on(&mut s, &alice, ABOVE, 2);
    for _ in 0..36 {
        console(&mut s, "give Alice stone 64");
    }
    alice.link.take();
    use_on(&mut s, &alice, ABOVE, 3);
    let window = window_opened(&alice).expect("the lectern opened");
    send_input(
        &mut s.server,
        &alice,
        PlayerInput::ContainerButtonClick { window, button: 3 },
    );
    assert_eq!(
        fired_kinds(&s.plugin),
        [
            (Phase::Settled, EventKind::PlayerTakeLecternBook),
            (Phase::Settled, EventKind::PlayerDropItem),
        ]
    );
    assert_eq!(s.server.items_on_ground(), 1);
}

#[test]
fn a_drop_settled_after_the_player_left_does_not_duplicate_the_item() {
    let mut s = scene_with(&["Alice"], vec![deciding(1, EventKind::PlayerDropItem)]);
    let alice = s.players[0].clone();
    console(&mut s, "give Alice diamond 3");
    drop_one(&mut s, &alice);
    let asked = last_asked(&s.plugin, EventKind::PlayerDropItem);
    s.server.handle(leave_of(&alice));
    assert_eq!(s.server.items_on_ground(), 1);
    rule_on(&mut s.server, 1, asked.id, false, None);
    assert_eq!(s.server.items_on_ground(), 1);
}

#[test]
fn a_finish_sent_before_its_left_click_settles_is_refused_by_a_cancel() {
    let mut s = scene_with(&["Alice"], vec![deciding(1, EventKind::PlayerInteract)]);
    let alice = s.players[0].clone();
    dig(&mut s, &alice, DigAction::Start, FLOOR);
    let asked = last_asked(&s.plugin, EventKind::PlayerInteract);
    dig(&mut s, &alice, DigAction::Finish, FLOOR);
    assert!(!block_is_air(&mut s, FLOOR));
    rule_on(&mut s.server, 1, asked.id, true, None);
    assert!(!block_is_air(&mut s, FLOOR));
}

#[test]
fn a_use_sent_before_its_block_click_settles_reads_the_click_memo() {
    let mut s = scene_with(&["Alice"], vec![deciding(1, EventKind::PlayerInteract)]);
    let alice = s.players[0].clone();
    console(&mut s, "give Alice bow 1");
    use_on(&mut s, &alice, FLOOR, 1);
    let asked = last_asked(&s.plugin, EventKind::PlayerInteract);
    send_input(
        &mut s.server,
        &alice,
        PlayerInput::UseItem {
            hand: Hand::Main,
            sequence: 2,
            rotation: Rotation::new(-90.0, 39.0),
        },
    );
    assert_eq!(asked_for(&s.plugin, EventKind::PlayerInteract).len(), 1);
    alice.link.take();
    rule_on(
        &mut s.server,
        1,
        asked.id,
        false,
        results(EventResult::Allow, EventResult::Deny),
    );
    assert_eq!(asked_for(&s.plugin, EventKind::PlayerInteract).len(), 1);
    assert!(
        alice
            .link
            .take()
            .iter()
            .any(|event| matches!(event, ClientEvent::BlockChangedAck(2)))
    );
}

#[test]
fn a_drop_sent_before_its_slot_change_settles_drops_from_the_new_slot() {
    let mut s = scene_with(&["Alice"], vec![deciding(1, EventKind::PlayerItemHeld)]);
    let alice = s.players[0].clone();
    console(&mut s, "give Alice diamond 1");
    console(&mut s, "give Alice gold_ingot 1");
    send_input(&mut s.server, &alice, PlayerInput::HeldSlot(1));
    let held = last_asked(&s.plugin, EventKind::PlayerItemHeld);
    drop_one(&mut s, &alice);
    rule_on(&mut s.server, 1, held.id, false, None);
    let slots = inventory(&mut s, &alice);
    assert_eq!(slots[0], Some(ItemSpec::of("minecraft:diamond", 1)));
    assert_eq!(slots[1], None);
    assert_eq!(s.server.items_on_ground(), 1);
}

#[test]
fn a_block_click_sent_before_its_slot_change_settles_uses_the_new_slot() {
    let mut s = scene_with(
        &["Alice"],
        vec![
            deciding(1, EventKind::PlayerItemHeld),
            observe(2, EventKind::PlayerInteract),
        ],
    );
    let alice = s.players[0].clone();
    console(&mut s, "give Alice stone 1");
    console(&mut s, "give Alice dirt 1");
    send_input(&mut s.server, &alice, PlayerInput::HeldSlot(1));
    let held = last_asked(&s.plugin, EventKind::PlayerItemHeld);
    use_on(&mut s, &alice, FLOOR, 1);
    rule_on(&mut s.server, 1, held.id, false, None);
    let heard = settled_for(&s.plugin, EventKind::PlayerInteract);
    assert_eq!(
        interact_of(&heard[0]).item,
        Some(ItemSpec::of("minecraft:dirt", 1))
    );
    match s.ask(Question::Block(ABOVE)) {
        Answer::Block(state) => assert_eq!(state.kind, "minecraft:dirt"),
        other => panic!("expected a block, got {other:?}"),
    }
}

fn refused_before_the_event(s: &mut Scene, player: &Player, pos: BlockPos) {
    player.link.take();
    use_on(s, player, pos, 7);
    assert!(settled_for(&s.plugin, EventKind::PlayerInteract).is_empty());
    let events = player.link.take();
    assert!(
        events
            .iter()
            .any(|event| matches!(event, ClientEvent::BlockUpdate { pos: at, .. } if *at == pos))
    );
    assert!(
        events
            .iter()
            .any(|event| matches!(event, ClientEvent::BlockChangedAck(7)))
    );
}

#[test]
fn a_right_click_out_of_reach_is_refused_before_the_event() {
    let mut s = scene_with(&["Alice"], vec![observe(1, EventKind::PlayerInteract)]);
    let alice = s.players[0].clone();
    console(&mut s, "give Alice stone 4");
    let far = BlockPos::new(12, 63, 0);
    refused_before_the_event(&mut s, &alice, far);
    assert!(block_is_air(&mut s, BlockPos::new(12, 64, 0)));
}

#[test]
fn a_right_click_on_the_protected_spawn_is_refused_before_the_event() {
    let mut server = configured(|config| config.spawn_protection = 16);
    let plugin = connect_with(
        &mut server,
        1,
        "tester",
        vec![observe(1, EventKind::PlayerInteract)],
    );
    let alice = join(&mut server, "Alice");
    let bob = join(&mut server, "Bob");
    let mut s = Scene {
        server,
        plugin,
        players: vec![alice.clone(), bob],
    };
    console(&mut s, "give Alice stone 4");
    refused_before_the_event(&mut s, &alice, FLOOR);
    assert!(block_is_air(&mut s, ABOVE));
}

fn updated(events: &[ClientEvent]) -> Vec<BlockPos> {
    events
        .iter()
        .filter_map(|event| match event {
            ClientEvent::BlockUpdate { pos, .. } => Some(*pos),
            _ => None,
        })
        .collect()
}

fn resent_inventory(events: &[ClientEvent]) -> bool {
    events
        .iter()
        .any(|event| matches!(event, ClientEvent::InventoryContent { .. }))
}

#[test]
fn a_denied_door_resends_its_other_half_and_the_inventory() {
    let mut s = scene_with(&["Alice"], vec![deciding(1, EventKind::PlayerInteract)]);
    let alice = s.players[0].clone();
    console(&mut s, "give Alice oak_door 1");
    use_on(&mut s, &alice, FLOOR, 1);
    let placed = last_asked(&s.plugin, EventKind::PlayerInteract);
    rule_on(&mut s.server, 1, placed.id, false, None);
    let upper = BlockPos::new(2, 65, 0);
    assert!(!block_is_air(&mut s, upper));
    alice.link.take();
    send_input(
        &mut s.server,
        &alice,
        PlayerInput::UseItemOn {
            hand: Hand::Main,
            pos: ABOVE,
            face: Direction::West,
            cursor: Vec3::new(0.0, 0.5, 0.5),
            inside_block: false,
            sequence: 2,
        },
    );
    let clicked = last_asked(&s.plugin, EventKind::PlayerInteract);
    rule_on(
        &mut s.server,
        1,
        clicked.id,
        false,
        results(EventResult::Deny, EventResult::Default),
    );
    let events = alice.link.take();
    assert!(updated(&events).contains(&upper), "{:?}", updated(&events));
    assert!(resent_inventory(&events));
}

#[test]
fn a_denied_click_with_a_door_in_hand_resends_the_two_blocks_it_would_fill() {
    let mut s = scene_with(&["Alice"], vec![deciding(1, EventKind::PlayerInteract)]);
    let alice = s.players[0].clone();
    console(&mut s, "give Alice oak_door 1");
    alice.link.take();
    use_on(&mut s, &alice, FLOOR, 1);
    let asked = last_asked(&s.plugin, EventKind::PlayerInteract);
    rule_on(&mut s.server, 1, asked.id, true, None);
    let events = alice.link.take();
    let updates = updated(&events);
    assert!(updates.contains(&BlockPos::new(2, 65, 0)), "{updates:?}");
    assert!(updates.contains(&ABOVE), "{updates:?}");
    assert!(block_is_air(&mut s, ABOVE));
}

#[test]
fn a_wear_whose_slot_changed_while_it_waited_is_dropped() {
    let mut s = scene_with(&["Alice"], vec![deciding(1, EventKind::PlayerItemDamage)]);
    let alice = s.players[0].clone();
    console(&mut s, "give Alice stone_sword 1");
    let cow = cow_near(&mut s);
    send_input(&mut s.server, &alice, PlayerInput::Attack(cow));
    let asked = last_asked(&s.plugin, EventKind::PlayerItemDamage);
    console(&mut s, "clear Alice");
    console(&mut s, "give Alice iron_sword 1");
    let fresh = damage_of(&mut s, &alice);
    rule_on(&mut s.server, 1, asked.id, false, None);
    assert_eq!(damage_of(&mut s, &alice), fresh);
    assert_eq!(
        inventory(&mut s, &alice)[0]
            .as_ref()
            .map(|item| item.key.clone()),
        Some("minecraft:iron_sword".to_owned())
    );
}

#[test]
fn a_book_take_whose_menu_closed_while_it_waited_leaves_the_book() {
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
    s.act(to(&alice, PlayerOrder::CloseInventory));
    rule_on(&mut s.server, 1, asked.id, false, None);
    assert!(
        !inventory(&mut s, &alice)
            .iter()
            .flatten()
            .any(|item| item.key == "minecraft:writable_book")
    );
    assert_eq!(s.server.items_on_ground(), 0);
}

#[test]
fn a_lectern_click_is_classified_as_spigot_does_and_writes_nothing_into_the_lectern() {
    let mut s = scene_with(
        &["Alice"],
        vec![
            deciding(1, EventKind::InventoryClick),
            observe(2, EventKind::InventoryClick),
        ],
    );
    let alice = s.players[0].clone();
    let window = lectern_with_a_book(&mut s, &alice);
    send_input(
        &mut s.server,
        &alice,
        PlayerInput::ContainerClick(rspigot_core::events::ContainerClick {
            container: window,
            state_id: 0,
            slot: 0,
            button: 0,
            input: rspigot_core::events::ClickInput::Pickup,
            changed: Vec::new(),
            carried: rspigot_core::events::HashedSlot::EMPTY,
        }),
    );
    let asked = last_asked(&s.plugin, EventKind::InventoryClick);
    let EventPayload::InventoryClick(click) = &asked.payload else {
        panic!("expected a click");
    };
    assert_eq!(
        click.action,
        rspigot_core::plugins::events::InventoryAction::PickupAll
    );
    assert_eq!(click.inventory.kind, "LECTERN");
    rule_on(
        &mut s.server,
        1,
        asked.id,
        false,
        Some(EventPatch::InventoryClick(
            rspigot_core::plugins::events::InventoryClickPatch {
                current_item: Some(None),
                ..Default::default()
            },
        )),
    );
    send_input(
        &mut s.server,
        &alice,
        PlayerInput::ContainerButtonClick { window, button: 3 },
    );
    assert!(
        inventory(&mut s, &alice)
            .iter()
            .flatten()
            .any(|item| item.key == "minecraft:writable_book")
    );
}
