mod common;

use common::*;
use rspigot_core::events::{InputState, PlayerInput};
use rspigot_core::plugins::events::Pose;
use rspigot_core::plugins::{Answer, EntityOrder, EventKind, EventPayload, Order, Question};
use rspigot_domain::{EntityId, Vec3};

fn poses(s: &Scene) -> Vec<(EntityId, Pose)> {
    fired(&s.plugin)
        .into_iter()
        .filter_map(|event| match event.payload {
            EventPayload::EntityPoseChange(change) => Some((change.entity.id, change.pose)),
            _ => None,
        })
        .collect()
}

fn sneak(s: &mut Scene, shift: bool) {
    let alice = s.players[0].clone();
    send_input(
        &mut s.server,
        &alice,
        PlayerInput::Input(InputState {
            shift,
            ..InputState::default()
        }),
    );
}

#[test]
fn a_sneaking_player_and_a_dying_mob_announce_their_new_pose_once_per_tick() {
    let mut s = scene_with(&["Alice"], vec![observe(1, EventKind::EntityPoseChange)]);
    let Answer::Spawned(cow) = s.ask(Question::Spawn {
        kind: "minecraft:cow".into(),
        position: Vec3::new(2.5, 64.0, 2.5),
    }) else {
        panic!("a cow");
    };
    s.tick(1);
    assert!(poses(&s).is_empty());
    sneak(&mut s, true);
    sneak(&mut s, false);
    sneak(&mut s, true);
    s.tick(1);
    let Answer::Player(alice) = s.ask(Question::Player(s.players[0].id)) else {
        panic!("Alice");
    };
    let alice = EntityId(alice.player.entity_id);
    assert_eq!(poses(&s), [(alice, Pose::Sneaking)]);
    s.act(Order::Entity {
        entity: cow.id,
        order: EntityOrder::Kill,
    });
    sneak(&mut s, false);
    s.tick(1);
    assert_eq!(
        poses(&s),
        [
            (alice, Pose::Sneaking),
            (cow.id, Pose::Dying),
            (alice, Pose::Standing)
        ]
    );
}
