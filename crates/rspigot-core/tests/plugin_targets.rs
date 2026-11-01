mod common;

use common::*;
use rspigot_core::events::{Direction, Inbound};
use rspigot_core::plugins::{
    Answer, BlockOrder, EntityOrder, EntityView, Order, PlayerOrder, Question, TargetHit,
};
use rspigot_domain::{BlockPos, Rotation, Vec3};

fn look_ahead(s: &mut Scene, player: &Player) {
    s.act(to(
        player,
        PlayerOrder::TeleportFacing(Vec3::new(0.5, 64.0, 0.5), Rotation::new(0.0, 0.0)),
    ));
}

fn put(s: &mut Scene, pos: BlockPos, order: BlockOrder) {
    s.act(Order::Block { pos, order });
}

fn set(key: &str) -> BlockOrder {
    BlockOrder::Set(key.into())
}

fn open_door(half: &str) -> BlockOrder {
    BlockOrder::SetWith {
        kind: "minecraft:oak_door".into(),
        properties: vec![
            ("facing".into(), "north".into()),
            ("open".into(), "true".into()),
            ("half".into(), half.into()),
        ],
    }
}

fn target_block(s: &mut Scene, player: &Player, max_distance: f64) -> Answer {
    s.ask(Question::TargetBlock {
        player: player.id,
        max_distance,
    })
}

fn target_entity(s: &mut Scene, player: &Player, max_distance: f64) -> Answer {
    s.ask(Question::TargetEntity {
        player: player.id,
        max_distance,
    })
}

fn north_face_of(pos: BlockPos, kind: &str) -> Answer {
    Answer::Target(Some(TargetHit::Block {
        pos,
        face: Direction::North,
        kind: kind.into(),
    }))
}

fn spawn(s: &mut Scene, kind: &str, position: Vec3) -> EntityView {
    let Answer::Spawned(view) = s.ask(Question::Spawn {
        kind: kind.into(),
        position,
    }) else {
        panic!("expected the spawned {kind}");
    };
    view
}

#[test]
fn target_block_returns_the_block_and_its_face() {
    let mut s = scene(&["Alice"]);
    let alice = s.players[0].clone();
    look_ahead(&mut s, &alice);
    let stone = BlockPos::new(0, 65, 3);
    put(&mut s, stone, set("minecraft:stone"));
    assert_eq!(
        target_block(&mut s, &alice, 10.0),
        north_face_of(stone, "minecraft:stone")
    );
    assert_eq!(target_block(&mut s, &alice, 2.0), Answer::Target(None));
    put(&mut s, stone, set("minecraft:air"));
    assert_eq!(target_block(&mut s, &alice, 10.0), Answer::Target(None));
    assert!(s.warnings().is_empty());
}

#[test]
fn the_ray_passes_an_open_doorway_and_stops_at_a_flower() {
    let mut s = scene(&["Alice"]);
    let alice = s.players[0].clone();
    look_ahead(&mut s, &alice);
    let (lower, upper) = (BlockPos::new(0, 64, 2), BlockPos::new(0, 65, 2));
    let wall = BlockPos::new(0, 65, 4);
    put(&mut s, lower, open_door("lower"));
    put(&mut s, upper, open_door("upper"));
    put(&mut s, wall, set("minecraft:stone"));
    assert_eq!(
        target_block(&mut s, &alice, 10.0),
        north_face_of(wall, "minecraft:stone")
    );

    put(&mut s, lower, set("minecraft:grass_block"));
    put(&mut s, upper, set("minecraft:poppy"));
    assert_eq!(
        target_block(&mut s, &alice, 10.0),
        north_face_of(upper, "minecraft:poppy")
    );

    put(&mut s, upper, set("minecraft:water"));
    assert_eq!(
        target_block(&mut s, &alice, 10.0),
        north_face_of(wall, "minecraft:stone")
    );
    assert!(s.warnings().is_empty());
}

#[test]
fn target_entity_returns_the_nearest_before_any_block() {
    let mut s = scene(&["Alice", "Bob"]);
    let (alice, bob) = (s.players[0].clone(), s.players[1].clone());
    s.act(to(&bob, PlayerOrder::Teleport(Vec3::new(8.5, 64.0, 0.5))));
    look_ahead(&mut s, &alice);
    let near = spawn(&mut s, "minecraft:skeleton", Vec3::new(0.5, 64.0, 3.5));
    assert_eq!(near.kind, "minecraft:skeleton");
    assert_eq!(
        target_entity(&mut s, &alice, 10.0),
        Answer::Target(Some(TargetHit::Entity(near.clone())))
    );

    s.act(Order::Entity {
        entity: near.id,
        order: EntityOrder::Remove,
    });
    spawn(&mut s, "minecraft:skeleton", Vec3::new(0.5, 64.0, 6.5));
    put(&mut s, BlockPos::new(0, 65, 4), set("minecraft:stone"));
    assert_eq!(target_entity(&mut s, &alice, 10.0), Answer::Target(None));

    s.act(to(&bob, PlayerOrder::Teleport(Vec3::new(0.5, 64.0, 2.5))));
    let Answer::Player(state) = s.ask(Question::Player(bob.id)) else {
        panic!("expected the state of Bob");
    };
    assert_eq!(
        target_entity(&mut s, &alice, 10.0),
        Answer::Target(Some(TargetHit::Player(state.player)))
    );
    assert!(s.warnings().is_empty());
}

#[test]
fn distance_is_checked() {
    let mut s = scene(&["Alice"]);
    let alice = s.players[0].clone();
    look_ahead(&mut s, &alice);
    let stone = BlockPos::new(0, 65, 3);
    put(&mut s, stone, set("minecraft:stone"));
    assert_eq!(
        target_block(&mut s, &alice, 500.0),
        north_face_of(stone, "minecraft:stone")
    );
    for refused in [-1.0, f64::NAN] {
        assert!(matches!(
            target_block(&mut s, &alice, refused),
            Answer::Refused(_)
        ));
        assert!(matches!(
            target_entity(&mut s, &alice, refused),
            Answer::Refused(_)
        ));
    }
    s.server.handle(Inbound::Leave { player: alice.id });
    s.server.deliver();
    assert_eq!(target_block(&mut s, &alice, 10.0), Answer::NotFound);
    assert_eq!(target_entity(&mut s, &alice, 10.0), Answer::NotFound);
    assert!(s.warnings().is_empty());
}

#[test]
fn a_shapeless_block_around_the_eyes_is_passed_through() {
    let mut s = scene(&["Alice"]);
    let alice = s.players[0].clone();
    look_ahead(&mut s, &alice);
    put(&mut s, BlockPos::new(0, 65, -1), set("minecraft:stone"));
    put(
        &mut s,
        BlockPos::new(0, 65, 0),
        BlockOrder::SetWith {
            kind: "minecraft:vine".into(),
            properties: vec![("north".into(), "true".into())],
        },
    );
    let stone = BlockPos::new(0, 65, 3);
    put(&mut s, stone, set("minecraft:stone"));
    assert_eq!(
        target_block(&mut s, &alice, 10.0),
        north_face_of(stone, "minecraft:stone")
    );
    let skeleton = spawn(&mut s, "minecraft:skeleton", Vec3::new(0.5, 64.0, 2.5));
    assert_eq!(
        target_entity(&mut s, &alice, 10.0),
        Answer::Target(Some(TargetHit::Entity(skeleton)))
    );
    assert!(s.warnings().is_empty());
}

#[test]
fn light_blocks_are_passed_through() {
    let mut s = scene(&["Alice"]);
    let alice = s.players[0].clone();
    look_ahead(&mut s, &alice);
    put(&mut s, BlockPos::new(0, 65, 2), set("minecraft:light"));
    let stone = BlockPos::new(0, 65, 4);
    put(&mut s, stone, set("minecraft:stone"));
    assert_eq!(
        target_block(&mut s, &alice, 10.0),
        north_face_of(stone, "minecraft:stone")
    );
    let skeleton = spawn(&mut s, "minecraft:skeleton", Vec3::new(0.5, 64.0, 3.5));
    assert_eq!(
        target_entity(&mut s, &alice, 10.0),
        Answer::Target(Some(TargetHit::Entity(skeleton)))
    );
    assert!(s.warnings().is_empty());
}
