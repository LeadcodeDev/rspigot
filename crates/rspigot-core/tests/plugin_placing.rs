mod common;

use common::*;
use rspigot_core::events::{ClientEvent, Direction, Hand, Inbound, PlayerInput};
use rspigot_core::plugins::events::{
    BlockCanBuild, BlockCanBuildPatch, BlockMultiPlace, BlockMultiPlacePatch, BlockPlace,
    BlockPlacePatch, EquipmentSlot,
};
use rspigot_core::plugins::{
    Answer, BlockOrder, EventKind, EventPatch, EventPayload, ItemSpec, Order, Question,
};
use rspigot_domain::{BlockPos, Vec3};

const FLOOR: BlockPos = BlockPos::new(2, 63, 0);
const ABOVE: BlockPos = BlockPos::new(2, 64, 0);
const TOP: BlockPos = BlockPos::new(2, 65, 0);

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

fn kind_at(s: &mut Scene, pos: BlockPos) -> String {
    match s.ask(Question::Block(pos)) {
        Answer::Block(state) => state.kind,
        other => panic!("expected a block, got {other:?}"),
    }
}

fn held(s: &mut Scene, player: &Player) -> Option<ItemSpec> {
    match s.ask(Question::Inventory(player.id)) {
        Answer::Inventory(inventory) => inventory.slots[0].clone(),
        other => panic!("expected the inventory, got {other:?}"),
    }
}

fn placing(handlers: Vec<rspigot_core::plugins::HandlerSpec>, item: &str) -> (Scene, Player) {
    let mut s = scene_with(&["Alice"], handlers);
    let alice = s.players[0].clone();
    console(&mut s, &format!("give Alice {item}"));
    stream_chunk_of(&mut s, FLOOR);
    alice.link.take();
    (s, alice)
}

fn packets(events: &[ClientEvent]) -> Vec<String> {
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
            ClientEvent::InventoryContent { .. } => Some("inventory".to_owned()),
            _ => None,
        })
        .collect()
}

fn place_of(fired: &Fired) -> BlockPlace {
    match &fired.payload {
        EventPayload::BlockPlace(event) => event.clone(),
        other => panic!("expected a place, got {other:?}"),
    }
}

#[test]
fn placing_a_block_fires_can_build_then_place_with_the_placed_block() {
    let (mut s, alice) = placing(
        vec![
            observe(1, EventKind::BlockCanBuild),
            observe(2, EventKind::BlockPlace),
        ],
        "stone 4",
    );
    use_on(&mut s, &alice, FLOOR, 1);
    let heard = fired(&s.plugin);
    assert_eq!(heard.len(), 2);
    let EventPayload::BlockCanBuild(BlockCanBuild {
        block,
        block_data,
        buildable,
        material,
        player,
    }) = &heard[0].payload
    else {
        panic!("expected a can build first, got {:?}", heard[0].payload);
    };
    assert_eq!(*block, ABOVE);
    assert_eq!(block_data.kind, "minecraft:stone");
    assert_eq!(material, "minecraft:stone");
    assert!(*buildable);
    assert_eq!(player.as_ref().map(|p| p.name.as_str()), Some("Alice"));
    let placed = place_of(&heard[1]);
    assert_eq!(placed.block, ABOVE);
    assert_eq!(placed.block_placed, ABOVE);
    assert_eq!(placed.block_against, FLOOR);
    assert_eq!(placed.block_replaced_state.pos, ABOVE);
    assert_eq!(placed.block_replaced_state.state.kind, "minecraft:air");
    assert!(placed.build);
    assert_eq!(placed.hand, EquipmentSlot::Hand);
    assert_eq!(placed.item_in_hand, ItemSpec::of("minecraft:stone", 4));
    assert_eq!(kind_at(&mut s, ABOVE), "minecraft:stone");
    assert_eq!(
        held(&mut s, &alice),
        Some(ItemSpec::of("minecraft:stone", 3))
    );
}

#[test]
fn a_cancelled_place_resends_the_captured_block_its_neighbours_and_the_inventory() {
    let (mut s, alice) = placing(vec![deciding(1, EventKind::BlockPlace)], "stone 4");
    use_on(&mut s, &alice, FLOOR, 3);
    assert_eq!(kind_at(&mut s, ABOVE), "minecraft:air");
    assert!(packets(&alice.link.take()).is_empty());
    let asked = last_asked(&s.plugin, EventKind::BlockPlace);
    assert_eq!(place_of(&asked).block, ABOVE);
    rule_on(&mut s.server, 1, asked.id, true, None);
    assert_eq!(
        packets(&alice.link.take()),
        [
            "inventory",
            "update 2 64 0 air",
            "update 2 63 0 block",
            "update 2 65 0 air",
            "update 2 64 -1 air",
            "update 2 64 1 air",
            "update 1 64 0 air",
            "update 3 64 0 air",
            "ack 3",
        ]
    );
    assert_eq!(kind_at(&mut s, ABOVE), "minecraft:air");
    assert_eq!(
        held(&mut s, &alice),
        Some(ItemSpec::of("minecraft:stone", 4))
    );
}

#[test]
fn an_accepted_place_updates_the_block_before_its_ack() {
    let (mut s, alice) = placing(vec![deciding(1, EventKind::BlockPlace)], "stone 4");
    use_on(&mut s, &alice, FLOOR, 3);
    let asked = last_asked(&s.plugin, EventKind::BlockPlace);
    rule_on(&mut s.server, 1, asked.id, false, None);
    let order = packets(&alice.link.take());
    assert_eq!(order[..2], ["update 2 64 0 block", "ack 3"]);
    assert_eq!(kind_at(&mut s, ABOVE), "minecraft:stone");
}

#[test]
fn clearing_build_refuses_the_place() {
    let (mut s, alice) = placing(vec![deciding(1, EventKind::BlockPlace)], "stone 4");
    use_on(&mut s, &alice, FLOOR, 3);
    let asked = last_asked(&s.plugin, EventKind::BlockPlace);
    rule_on(
        &mut s.server,
        1,
        asked.id,
        false,
        Some(EventPatch::BlockPlace(BlockPlacePatch {
            build: Some(false),
        })),
    );
    assert_eq!(kind_at(&mut s, ABOVE), "minecraft:air");
    assert_eq!(
        held(&mut s, &alice),
        Some(ItemSpec::of("minecraft:stone", 4))
    );
}

#[test]
fn a_block_changed_while_its_place_waits_is_refused() {
    let (mut s, alice) = placing(vec![deciding(1, EventKind::BlockPlace)], "stone 4");
    use_on(&mut s, &alice, FLOOR, 3);
    let asked = last_asked(&s.plugin, EventKind::BlockPlace);
    s.act(Order::Block {
        pos: ABOVE,
        order: BlockOrder::Set("minecraft:glass".to_owned()),
    });
    rule_on(&mut s.server, 1, asked.id, false, None);
    assert_eq!(kind_at(&mut s, ABOVE), "minecraft:glass");
    assert_eq!(
        held(&mut s, &alice),
        Some(ItemSpec::of("minecraft:stone", 4))
    );
}

#[test]
fn clearing_buildable_refuses_the_place_before_it_is_asked() {
    let (mut s, alice) = placing(
        vec![
            deciding(1, EventKind::BlockCanBuild),
            observe(2, EventKind::BlockPlace),
        ],
        "stone 4",
    );
    use_on(&mut s, &alice, FLOOR, 3);
    let asked = last_asked(&s.plugin, EventKind::BlockCanBuild);
    rule_on(
        &mut s.server,
        1,
        asked.id,
        false,
        Some(EventPatch::BlockCanBuild(BlockCanBuildPatch {
            buildable: Some(false),
        })),
    );
    assert_eq!(kind_at(&mut s, ABOVE), "minecraft:air");
    assert!(settled_for(&s.plugin, EventKind::BlockPlace).is_empty());
    assert_eq!(packets(&alice.link.take()), ["ack 3"]);
}

#[test]
fn setting_buildable_places_a_block_where_the_player_stands() {
    let (mut s, alice) = placing(vec![deciding(1, EventKind::BlockCanBuild)], "stone 4");
    let under = BlockPos::new(0, 63, 0);
    let feet = BlockPos::new(0, 64, 0);
    use_on(&mut s, &alice, under, 3);
    let asked = last_asked(&s.plugin, EventKind::BlockCanBuild);
    match &asked.payload {
        EventPayload::BlockCanBuild(event) => assert!(!event.buildable),
        other => panic!("expected a can build, got {other:?}"),
    }
    rule_on(&mut s.server, 1, asked.id, false, None);
    assert_eq!(kind_at(&mut s, feet), "minecraft:air");
    use_on(&mut s, &alice, under, 4);
    let asked = last_asked(&s.plugin, EventKind::BlockCanBuild);
    rule_on(
        &mut s.server,
        1,
        asked.id,
        false,
        Some(EventPatch::BlockCanBuild(BlockCanBuildPatch {
            buildable: Some(true),
        })),
    );
    assert_eq!(kind_at(&mut s, feet), "minecraft:stone");
}

#[test]
fn a_door_is_a_multi_place_in_capture_order_heard_by_place_handlers_too() {
    let (mut s, alice) = placing(
        vec![
            deciding(1, EventKind::BlockMultiPlace),
            observe(2, EventKind::BlockPlace),
        ],
        "oak_door 2",
    );
    use_on(&mut s, &alice, FLOOR, 5);
    let asked = last_asked(&s.plugin, EventKind::BlockMultiPlace);
    let EventPayload::BlockMultiPlace(BlockMultiPlace {
        block,
        block_placed,
        block_against,
        replaced_block_states,
        hand,
        item_in_hand,
        ..
    }) = &asked.payload
    else {
        panic!("expected a multi place");
    };
    assert_eq!((*block, *block_placed, *block_against), (TOP, TOP, FLOOR));
    let replaced: Vec<BlockPos> = replaced_block_states
        .iter()
        .map(|state| state.pos)
        .collect();
    assert_eq!(replaced, [TOP, ABOVE]);
    assert_eq!(*hand, EquipmentSlot::Hand);
    assert_eq!(*item_in_hand, ItemSpec::of("minecraft:oak_door", 2));
    assert_eq!(kind_at(&mut s, ABOVE), "minecraft:air");
    alice.link.take();
    rule_on(&mut s.server, 1, asked.id, true, None);
    let resent = packets(&alice.link.take());
    assert_eq!(
        resent[..3],
        ["inventory", "update 2 65 0 air", "update 2 64 0 air"]
    );
    assert_eq!(resent.last().map(String::as_str), Some("ack 5"));
    assert_eq!(kind_at(&mut s, ABOVE), "minecraft:air");
    assert_eq!(kind_at(&mut s, TOP), "minecraft:air");
    let heard = settled_for(&s.plugin, EventKind::BlockMultiPlace);
    assert!(heard[0].cancelled);
    use_on(&mut s, &alice, FLOOR, 6);
    let asked = last_asked(&s.plugin, EventKind::BlockMultiPlace);
    rule_on(&mut s.server, 1, asked.id, false, None);
    assert_eq!(kind_at(&mut s, ABOVE), "minecraft:oak_door");
    assert_eq!(kind_at(&mut s, TOP), "minecraft:oak_door");
}

#[test]
fn tilling_fires_a_place_and_a_cancel_keeps_the_dirt_and_the_hoe() {
    let (mut s, alice) = placing(
        vec![
            deciding(1, EventKind::BlockPlace),
            observe(2, EventKind::PlayerItemDamage),
        ],
        "iron_hoe",
    );
    s.act(Order::Block {
        pos: FLOOR,
        order: BlockOrder::Set("minecraft:dirt".to_owned()),
    });
    use_on(&mut s, &alice, FLOOR, 7);
    let asked = last_asked(&s.plugin, EventKind::BlockPlace);
    let placed = place_of(&asked);
    assert_eq!(placed.block, FLOOR);
    assert_eq!(placed.block_against, FLOOR);
    assert_eq!(placed.block_replaced_state.state.kind, "minecraft:dirt");
    assert_eq!(placed.item_in_hand.key, "minecraft:iron_hoe");
    rule_on(&mut s.server, 1, asked.id, true, None);
    assert_eq!(kind_at(&mut s, FLOOR), "minecraft:dirt");
    assert!(settled_for(&s.plugin, EventKind::PlayerItemDamage).is_empty());
    use_on(&mut s, &alice, FLOOR, 8);
    let asked = last_asked(&s.plugin, EventKind::BlockPlace);
    rule_on(&mut s.server, 1, asked.id, false, None);
    assert_eq!(kind_at(&mut s, FLOOR), "minecraft:farmland");
    assert_eq!(settled_for(&s.plugin, EventKind::PlayerItemDamage).len(), 1);
}

fn half_of(snapshot: &rspigot_core::plugins::BlockSnapshot) -> (BlockPos, String, Option<String>) {
    let half = snapshot
        .state
        .properties
        .iter()
        .find(|(name, _)| name == "half")
        .map(|(_, value)| value.clone());
    (snapshot.pos, snapshot.state.kind.clone(), half)
}

#[test]
fn a_place_carries_the_state_its_block_will_get() {
    let (mut s, alice) = placing(vec![deciding(1, EventKind::BlockPlace)], "stone 4");
    use_on(&mut s, &alice, FLOOR, 1);
    let placed = place_of(&last_asked(&s.plugin, EventKind::BlockPlace));
    assert_eq!(placed.placed_state.pos, ABOVE);
    assert_eq!(placed.placed_state.state.kind, "minecraft:stone");
    assert_eq!(placed.block_replaced_state.state.kind, "minecraft:air");
    assert_eq!(kind_at(&mut s, ABOVE), "minecraft:air");
}

#[test]
fn a_multi_place_carries_the_states_of_both_door_halves() {
    let (mut s, alice) = placing(vec![deciding(1, EventKind::BlockMultiPlace)], "oak_door 1");
    use_on(&mut s, &alice, FLOOR, 1);
    let asked = last_asked(&s.plugin, EventKind::BlockMultiPlace);
    let EventPayload::BlockMultiPlace(BlockMultiPlace {
        placed_state,
        placed_states,
        ..
    }) = &asked.payload
    else {
        panic!("expected a multi place");
    };
    let door = "minecraft:oak_door".to_owned();
    let halves: Vec<_> = placed_states.iter().map(half_of).collect();
    assert_eq!(
        halves,
        [
            (TOP, door.clone(), Some("upper".to_owned())),
            (ABOVE, door, Some("lower".to_owned())),
        ]
    );
    assert_eq!(placed_state, &placed_states[0]);
}

#[test]
fn a_tilled_place_carries_the_farmland_state() {
    let (mut s, alice) = placing(vec![deciding(1, EventKind::BlockPlace)], "iron_hoe");
    s.act(Order::Block {
        pos: FLOOR,
        order: BlockOrder::Set("minecraft:dirt".to_owned()),
    });
    use_on(&mut s, &alice, FLOOR, 7);
    let placed = place_of(&last_asked(&s.plugin, EventKind::BlockPlace));
    assert_eq!(placed.placed_state.pos, FLOOR);
    assert_eq!(placed.placed_state.state.kind, "minecraft:farmland");
    assert_eq!(kind_at(&mut s, FLOOR), "minecraft:dirt");
}

#[test]
fn the_roots_of_a_tilled_rooted_dirt_drop_only_when_the_place_applies() {
    let (mut s, alice) = placing(vec![deciding(1, EventKind::BlockPlace)], "iron_hoe");
    s.act(Order::Block {
        pos: FLOOR,
        order: BlockOrder::Set("minecraft:rooted_dirt".to_owned()),
    });
    use_on(&mut s, &alice, FLOOR, 7);
    let asked = last_asked(&s.plugin, EventKind::BlockPlace);
    assert_eq!(s.server.items_on_ground(), 0);
    rule_on(&mut s.server, 1, asked.id, true, None);
    assert_eq!(s.server.items_on_ground(), 0);
    assert_eq!(kind_at(&mut s, FLOOR), "minecraft:rooted_dirt");
    use_on(&mut s, &alice, FLOOR, 8);
    let asked = last_asked(&s.plugin, EventKind::BlockPlace);
    assert_eq!(s.server.items_on_ground(), 0);
    rule_on(&mut s.server, 1, asked.id, false, None);
    assert_eq!(kind_at(&mut s, FLOOR), "minecraft:dirt");
    assert_eq!(s.server.items_on_ground(), 1);
}

#[test]
fn spawn_protection_tests_the_clicked_block_not_the_placed_one() {
    let mut server = configured(|config| config.spawn_protection = 1);
    let plugin = connect_with(
        &mut server,
        1,
        "tester",
        vec![
            deciding(1, EventKind::BlockCanBuild),
            observe(2, EventKind::BlockPlace),
        ],
    );
    let alice = join(&mut server, "Alice");
    let mut s = Scene {
        server,
        plugin,
        players: vec![alice.clone()],
    };
    console(&mut s, "give Alice stone 4");
    let inside = BlockPos::new(1, 64, 0);
    let outside = BlockPos::new(2, 64, 0);
    s.act(Order::Block {
        pos: inside,
        order: BlockOrder::Set("minecraft:stone".to_owned()),
    });
    stream_chunk_of(&mut s, inside);
    send_input(
        &mut s.server,
        &alice,
        PlayerInput::UseItemOn {
            hand: Hand::Main,
            pos: inside,
            face: Direction::East,
            cursor: Vec3::new(1.0, 0.5, 0.5),
            inside_block: false,
            sequence: 1,
        },
    );
    let asked = last_asked(&s.plugin, EventKind::BlockCanBuild);
    let _bob = join(&mut s.server, "Bob");
    rule_on(&mut s.server, 1, asked.id, false, None);
    let placed = place_of(&settled_for(&s.plugin, EventKind::BlockPlace)[0]);
    assert_eq!(placed.block, outside);
    assert!(!placed.build);
    assert_eq!(kind_at(&mut s, outside), "minecraft:air");
}

#[test]
fn a_tool_use_resends_the_inventory_and_block_before_its_ack_and_acks_after_an_accepted_change() {
    let (mut s, alice) = placing(vec![deciding(1, EventKind::BlockPlace)], "iron_hoe");
    s.act(Order::Block {
        pos: FLOOR,
        order: BlockOrder::Set("minecraft:dirt".to_owned()),
    });
    alice.link.take();
    use_on(&mut s, &alice, FLOOR, 7);
    let asked = last_asked(&s.plugin, EventKind::BlockPlace);
    assert!(packets(&alice.link.take()).is_empty());
    rule_on(&mut s.server, 1, asked.id, true, None);
    let refused = packets(&alice.link.take());
    assert_eq!(refused[..2], ["inventory", "update 2 63 0 block"]);
    assert_eq!(refused.last().map(String::as_str), Some("ack 7"));
    use_on(&mut s, &alice, FLOOR, 8);
    let asked = last_asked(&s.plugin, EventKind::BlockPlace);
    rule_on(&mut s.server, 1, asked.id, false, None);
    let accepted = packets(&alice.link.take());
    assert_eq!(accepted, ["update 2 63 0 block", "ack 8"]);
}

#[test]
fn clearing_build_on_a_multi_place_refuses_both_halves() {
    let (mut s, alice) = placing(vec![deciding(1, EventKind::BlockMultiPlace)], "oak_door 1");
    use_on(&mut s, &alice, FLOOR, 5);
    let asked = last_asked(&s.plugin, EventKind::BlockMultiPlace);
    alice.link.take();
    rule_on(
        &mut s.server,
        1,
        asked.id,
        false,
        Some(EventPatch::BlockMultiPlace(BlockMultiPlacePatch {
            build: Some(false),
        })),
    );
    assert_eq!(kind_at(&mut s, ABOVE), "minecraft:air");
    assert_eq!(kind_at(&mut s, TOP), "minecraft:air");
    let resent = packets(&alice.link.take());
    assert_eq!(
        resent[..3],
        ["inventory", "update 2 65 0 air", "update 2 64 0 air"]
    );
    assert_eq!(resent.last().map(String::as_str), Some("ack 5"));
    assert_eq!(
        held(&mut s, &alice),
        Some(ItemSpec::of("minecraft:oak_door", 1))
    );
}
