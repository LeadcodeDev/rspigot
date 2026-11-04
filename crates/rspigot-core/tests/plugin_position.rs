mod common;

use common::*;
use rspigot_core::events::{ClientEvent, Inbound, PlayerInput};
use rspigot_core::plugins::events::{
    EntityDeathPatch, PlayerDeath, PlayerDeathPatch, PlayerMovePatch, PlayerRespawn,
    PlayerRespawnPatch, PlayerRespawnReason, PlayerTeleport, PlayerTeleportCause,
    PlayerTeleportPatch, PlayerVelocity, PlayerVelocityPatch,
};
use rspigot_core::plugins::{EventKind, EventPatch, EventPayload, ItemSpec, Location, PlayerOrder};
use rspigot_domain::{EntityId, Rotation, Vec3};

const SPAWN: Vec3 = Vec3::new(0.5, 64.0, 0.5);

fn at(x: f64, z: f64) -> Vec3 {
    Vec3::new(x, 64.0, z)
}

fn location(position: Vec3) -> Location {
    Location {
        world: "minecraft:overworld".to_owned(),
        position,
        rotation: Rotation::default(),
    }
}

fn console(s: &mut Scene, line: &str) {
    s.server.handle(Inbound::Console(line.to_owned()));
    s.server.deliver();
}

fn teleports(player: &Player) -> Vec<(Vec3, Rotation)> {
    player
        .link
        .take()
        .into_iter()
        .filter_map(|event| match event {
            ClientEvent::Teleport {
                position, rotation, ..
            } => Some((position, rotation)),
            _ => None,
        })
        .collect()
}

#[test]
fn a_command_teleport_fires_with_its_cause_and_can_be_cancelled() {
    let mut s = scene_with(&["Alice"], vec![deciding(1, EventKind::PlayerTeleport)]);
    let alice = s.players[0].clone();
    console(&mut s, "tp Alice 10.5 64 0.5");
    assert_eq!(s.server.player_location(alice.id), Some(SPAWN));
    let asked = last_asked(&s.plugin, EventKind::PlayerTeleport);
    let EventPayload::PlayerTeleport(PlayerTeleport {
        cause, from, to, ..
    }) = &asked.payload
    else {
        panic!("expected a teleport");
    };
    assert_eq!(*cause, PlayerTeleportCause::Command);
    assert_eq!(from.position, SPAWN);
    assert_eq!(to.as_ref().map(|to| to.position), Some(at(10.5, 0.5)));
    rule_on(&mut s.server, 1, asked.id, true, None);
    assert_eq!(s.server.player_location(alice.id), Some(SPAWN));
    assert!(teleports(&alice).is_empty());
}

#[test]
fn a_changed_teleport_destination_and_rotation_apply() {
    let mut s = scene_with(&["Alice"], vec![deciding(1, EventKind::PlayerTeleport)]);
    let alice = s.players[0].clone();
    console(&mut s, "tp Alice 10.5 64 0.5");
    let asked = last_asked(&s.plugin, EventKind::PlayerTeleport);
    let mut elsewhere = location(at(3.5, 7.5));
    elsewhere.rotation = Rotation::new(90.0, 10.0);
    rule_on(
        &mut s.server,
        1,
        asked.id,
        false,
        Some(EventPatch::PlayerTeleport(PlayerTeleportPatch {
            from: None,
            to: Some(elsewhere),
        })),
    );
    assert_eq!(s.server.player_location(alice.id), Some(at(3.5, 7.5)));
    assert_eq!(
        teleports(&alice),
        [(at(3.5, 7.5), Rotation::new(90.0, 10.0))]
    );
}

#[test]
fn a_plugin_teleport_reaches_the_calling_plugin_as_a_plugin_cause() {
    let mut s = scene_with(&["Alice"], vec![deciding(1, EventKind::PlayerTeleport)]);
    let alice = s.players[0].clone();
    s.act(to(
        &alice,
        PlayerOrder::TeleportFacing(at(5.5, 0.5), Rotation::new(45.0, 0.0)),
    ));
    let asked = last_asked(&s.plugin, EventKind::PlayerTeleport);
    let EventPayload::PlayerTeleport(PlayerTeleport { cause, to, .. }) = &asked.payload else {
        panic!("expected a teleport");
    };
    assert_eq!(*cause, PlayerTeleportCause::Plugin);
    assert_eq!(
        to.as_ref().map(|to| to.rotation),
        Some(Rotation::new(45.0, 0.0))
    );
    rule_on(&mut s.server, 1, asked.id, false, None);
    assert_eq!(s.server.player_location(alice.id), Some(at(5.5, 0.5)));
    assert_eq!(
        teleports(&alice),
        [(at(5.5, 0.5), Rotation::new(45.0, 0.0))]
    );
}

#[test]
fn a_respawn_does_not_fire_a_teleport() {
    let mut s = scene_with(&["Alice"], vec![observe(1, EventKind::PlayerTeleport)]);
    let alice = s.players[0].clone();
    s.act(to(&alice, PlayerOrder::SetHealth(0.0)));
    send_input(&mut s.server, &alice, PlayerInput::Respawn);
    assert_eq!(s.server.player_is_dead(alice.id), Some(false));
    assert!(settled_for(&s.plugin, EventKind::PlayerTeleport).is_empty());
}

#[test]
fn a_changed_move_destination_fires_a_plugin_teleport_whose_cancel_keeps_the_move() {
    let mut s = scene_with(
        &["Alice"],
        vec![decide(1), deciding(2, EventKind::PlayerTeleport)],
    );
    let alice = s.players[0].clone();
    send_input(
        &mut s.server,
        &alice,
        PlayerInput::Move {
            position: Some(at(1.5, 0.5)),
            rotation: None,
            on_ground: true,
        },
    );
    let moved = last_asked(&s.plugin, EventKind::PlayerMove);
    rule_on(
        &mut s.server,
        1,
        moved.id,
        false,
        Some(EventPatch::PlayerMove(PlayerMovePatch {
            to: Some(location(at(8.5, 0.5))),
        })),
    );
    let teleport = last_asked(&s.plugin, EventKind::PlayerTeleport);
    let EventPayload::PlayerTeleport(PlayerTeleport { cause, to, .. }) = &teleport.payload else {
        panic!("expected a teleport");
    };
    assert_eq!(*cause, PlayerTeleportCause::Plugin);
    assert_eq!(to.as_ref().map(|to| to.position), Some(at(8.5, 0.5)));
    assert_eq!(s.server.player_location(alice.id), Some(SPAWN));
    judge(&mut s.server, 1, teleport.id, 2, true, None);
    assert_eq!(s.server.player_location(alice.id), Some(at(1.5, 0.5)));
}

fn velocities(player: &Player, entity: EntityId) -> Vec<Vec3> {
    player
        .link
        .take()
        .into_iter()
        .filter_map(|event| match event {
            ClientEvent::EntityVelocity {
                entity: moved,
                velocity,
            } if moved == entity => Some(velocity),
            _ => None,
        })
        .collect()
}

fn push(s: &mut Scene, player: &Player, y: f64) {
    s.act(to(player, PlayerOrder::SetVelocity(Vec3::new(0.0, y, 0.0))));
}

#[test]
fn a_velocity_without_a_decider_is_sent_at_once_and_only_observed() {
    let mut s = scene_with(&["Alice"], vec![observe(1, EventKind::PlayerVelocity)]);
    let alice = s.players[0].clone();
    let entity = s.server.player_entity_id(alice.id).expect("an entity");
    push(&mut s, &alice, 0.5);
    s.tick(1);
    assert_eq!(velocities(&alice, entity), [Vec3::new(0.0, 0.5, 0.0)]);
    let heard = settled_for(&s.plugin, EventKind::PlayerVelocity);
    assert_eq!(heard.len(), 1);
    assert_eq!(
        heard[0].payload,
        EventPayload::PlayerVelocity(PlayerVelocity {
            player: velocity_player(&heard[0].payload),
            velocity: Vec3::new(0.0, 0.5, 0.0),
        })
    );
    s.tick(3);
    assert_eq!(settled_for(&s.plugin, EventKind::PlayerVelocity).len(), 1);
}

fn velocity_player(payload: &EventPayload) -> rspigot_core::plugins::PlayerSnapshot {
    match payload {
        EventPayload::PlayerVelocity(PlayerVelocity { player, .. }) => player.clone(),
        other => panic!("expected a velocity, got {other:?}"),
    }
}

#[test]
fn a_cancelled_velocity_is_asked_again_but_never_twice_at_once() {
    let mut s = scene_with(&["Alice"], vec![deciding(1, EventKind::PlayerVelocity)]);
    let alice = s.players[0].clone();
    let entity = s.server.player_entity_id(alice.id).expect("an entity");
    push(&mut s, &alice, 0.5);
    s.server.tick();
    s.server.tick();
    s.server.deliver();
    let asked = asked_for(&s.plugin, EventKind::PlayerVelocity);
    assert_eq!(asked.len(), 1);
    rule_on(&mut s.server, 1, asked[0].id, true, None);
    assert!(velocities(&alice, entity).is_empty());
    s.server.tick();
    s.server.deliver();
    let asked = asked_for(&s.plugin, EventKind::PlayerVelocity);
    assert_eq!(asked.len(), 2);
    rule_on(
        &mut s.server,
        1,
        asked[1].id,
        false,
        Some(EventPatch::PlayerVelocity(PlayerVelocityPatch {
            velocity: Some(Vec3::new(0.0, 1.5, 0.0)),
        })),
    );
    assert_eq!(velocities(&alice, entity), [Vec3::new(0.0, 1.5, 0.0)]);
    s.tick(2);
    assert_eq!(asked_for(&s.plugin, EventKind::PlayerVelocity).len(), 2);
}

#[test]
fn a_hit_player_is_asked_with_the_velocity_before_the_hit() {
    let mut s = scene_with(
        &["Alice", "Bob"],
        vec![deciding(1, EventKind::PlayerVelocity)],
    );
    let (alice, bob) = (s.players[0].clone(), s.players[1].clone());
    let entity = s.server.player_entity_id(bob.id).expect("an entity");
    send_input(
        &mut s.server,
        &bob,
        PlayerInput::Move {
            position: Some(at(1.5, 0.5)),
            rotation: None,
            on_ground: true,
        },
    );
    s.tick(30);
    bob.link.take();
    send_input(&mut s.server, &alice, PlayerInput::Attack(entity));
    let asked = last_asked(&s.plugin, EventKind::PlayerVelocity);
    let EventPayload::PlayerVelocity(PlayerVelocity { velocity, player }) = &asked.payload else {
        panic!("expected a velocity");
    };
    assert_eq!(player.id, bob.id);
    assert_eq!(*velocity, Vec3::ZERO);
    assert!(velocities(&bob, entity).is_empty());
    rule_on(
        &mut s.server,
        1,
        asked.id,
        false,
        Some(EventPatch::PlayerVelocity(PlayerVelocityPatch {
            velocity: Some(Vec3::new(0.0, 2.0, 0.0)),
        })),
    );
    assert_eq!(velocities(&bob, entity), [Vec3::new(0.0, 2.0, 0.0)]);
}

fn kill(s: &mut Scene, player: &Player) {
    s.act(to(player, PlayerOrder::SetHealth(0.0)));
    assert_eq!(s.server.player_is_dead(player.id), Some(true));
}

#[test]
fn a_respawn_decider_moves_the_respawn_location() {
    let mut s = scene_with(&["Alice"], vec![deciding(1, EventKind::PlayerRespawn)]);
    let alice = s.players[0].clone();
    kill(&mut s, &alice);
    send_input(&mut s.server, &alice, PlayerInput::Respawn);
    assert_eq!(s.server.player_is_dead(alice.id), Some(true));
    let asked = last_asked(&s.plugin, EventKind::PlayerRespawn);
    let EventPayload::PlayerRespawn(PlayerRespawn {
        respawn_location,
        respawn_reason,
        bed_spawn,
        anchor_spawn,
        ..
    }) = &asked.payload
    else {
        panic!("expected a respawn");
    };
    assert_eq!(respawn_location.position, SPAWN);
    assert_eq!(*respawn_reason, PlayerRespawnReason::Death);
    assert!(!bed_spawn && !anchor_spawn);
    let mut there = location(at(20.5, 4.5));
    there.rotation = Rotation::new(180.0, 0.0);
    send_input(&mut s.server, &alice, PlayerInput::Respawn);
    alice.link.take();
    rule_on(
        &mut s.server,
        1,
        asked.id,
        false,
        Some(EventPatch::PlayerRespawn(PlayerRespawnPatch {
            respawn_location: Some(there),
        })),
    );
    assert_eq!(s.server.player_is_dead(alice.id), Some(false));
    assert_eq!(s.server.player_location(alice.id), Some(at(20.5, 4.5)));
    let second = last_asked(&s.plugin, EventKind::PlayerRespawn);
    rule_on(&mut s.server, 1, second.id, false, None);
    let respawns = alice
        .link
        .take()
        .into_iter()
        .filter(|event| matches!(event, ClientEvent::Respawn { .. }))
        .count();
    assert_eq!(respawns, 1);
    assert_eq!(s.server.player_location(alice.id), Some(at(20.5, 4.5)));
}

#[test]
fn a_plugin_respawn_carries_the_plugin_reason() {
    let mut s = scene_with(&["Alice"], vec![observe(1, EventKind::PlayerRespawn)]);
    let alice = s.players[0].clone();
    kill(&mut s, &alice);
    s.act(to(&alice, PlayerOrder::Respawn));
    let heard = settled_for(&s.plugin, EventKind::PlayerRespawn);
    let EventPayload::PlayerRespawn(PlayerRespawn { respawn_reason, .. }) = &heard[0].payload
    else {
        panic!("expected a respawn");
    };
    assert_eq!(*respawn_reason, PlayerRespawnReason::Plugin);
}

fn give(s: &mut Scene, line: &str) {
    console(s, line);
}

fn death_of(s: &Scene) -> (u64, PlayerDeath) {
    let asked = last_asked(&s.plugin, EventKind::PlayerDeath);
    match asked.payload {
        EventPayload::PlayerDeath(death) => (asked.id, death),
        other => panic!("expected a death, got {other:?}"),
    }
}

#[test]
fn a_death_describes_its_drops_and_waits_for_its_deciders() {
    let mut s = scene_with(&["Alice", "Bob"], vec![deciding(1, EventKind::PlayerDeath)]);
    let (alice, bob) = (s.players[0].clone(), s.players[1].clone());
    give(&mut s, "give Alice diamond 5");
    s.act(to(&alice, PlayerOrder::GiveLevels(30)));
    kill(&mut s, &alice);
    assert_eq!(s.server.items_on_ground(), 0);
    let (event, death) = death_of(&s);
    assert_eq!(death.drops, [ItemSpec::of("minecraft:diamond", 5)]);
    assert_eq!(death.entity.id, alice.id);
    assert_eq!(death.entity_type, "minecraft:player");
    assert!(!death.keep_inventory && !death.keep_level);
    assert_eq!(
        (death.new_exp, death.new_level, death.new_total_exp),
        (0, 0, 0)
    );
    assert!(
        death
            .death_message
            .as_deref()
            .is_some_and(|m| m.contains("Alice")),
        "{:?}",
        death.death_message
    );
    bob.link.take();
    rule_on(
        &mut s.server,
        1,
        event,
        false,
        Some(EventPatch::PlayerDeath(PlayerDeathPatch {
            death_message: Some(Some("§cAlice is no more".to_owned())),
            ..PlayerDeathPatch::default()
        })),
    );
    assert_eq!(s.server.items_on_ground(), 1);
    assert_eq!(system_texts(&bob), ["Alice is no more"]);
}

#[test]
fn a_death_with_cleared_drops_and_no_message_drops_nothing_and_says_nothing() {
    let mut s = scene_with(&["Alice", "Bob"], vec![deciding(1, EventKind::PlayerDeath)]);
    let (alice, bob) = (s.players[0].clone(), s.players[1].clone());
    give(&mut s, "give Alice diamond 5");
    kill(&mut s, &alice);
    let (event, _) = death_of(&s);
    bob.link.take();
    alice.link.take();
    rule_on(
        &mut s.server,
        1,
        event,
        false,
        Some(EventPatch::PlayerDeath(PlayerDeathPatch {
            death_message: Some(None),
            drops: Some(Vec::new()),
            dropped_exp: Some(0),
            ..PlayerDeathPatch::default()
        })),
    );
    assert_eq!(s.server.items_on_ground(), 0);
    assert!(system_texts(&bob).is_empty());
    let kill_message = alice.link.take().into_iter().find_map(|event| match event {
        ClientEvent::CombatKill { message, .. } => Some(message),
        _ => None,
    });
    assert_eq!(
        kill_message.map(|m| rspigot_core::console::console_text(&m)),
        Some(String::new())
    );
}

#[test]
fn keeping_the_inventory_without_clearing_the_drops_duplicates_them_as_in_spigot() {
    let mut s = scene_with(&["Alice"], vec![deciding(1, EventKind::PlayerDeath)]);
    let alice = s.players[0].clone();
    give(&mut s, "give Alice diamond 5");
    kill(&mut s, &alice);
    let (event, _) = death_of(&s);
    rule_on(
        &mut s.server,
        1,
        event,
        false,
        Some(EventPatch::PlayerDeath(PlayerDeathPatch {
            keep_inventory: Some(true),
            ..PlayerDeathPatch::default()
        })),
    );
    assert_eq!(s.server.items_on_ground(), 1);
    let rspigot_core::plugins::Answer::Inventory(inventory) =
        s.ask(rspigot_core::plugins::Question::Inventory(alice.id))
    else {
        panic!("expected the inventory");
    };
    assert!(
        inventory
            .slots
            .iter()
            .flatten()
            .any(|item| item.key == "minecraft:diamond")
    );
}

#[test]
fn the_new_level_waits_for_the_respawn_and_keep_level_ignores_it() {
    for (keep_level, expected) in [(false, 7), (true, 30)] {
        let mut s = scene_with(&["Alice"], vec![deciding(1, EventKind::PlayerDeath)]);
        let alice = s.players[0].clone();
        s.act(to(&alice, PlayerOrder::GiveLevels(30)));
        kill(&mut s, &alice);
        let (event, _) = death_of(&s);
        rule_on(
            &mut s.server,
            1,
            event,
            false,
            Some(EventPatch::PlayerDeath(PlayerDeathPatch {
                keep_level: Some(keep_level),
                new_level: Some(7),
                new_total_exp: Some(91),
                ..PlayerDeathPatch::default()
            })),
        );
        assert_eq!(
            s.server.player_experience(alice.id).map(|xp| xp.0),
            Some(30)
        );
        send_input(&mut s.server, &alice, PlayerInput::Respawn);
        assert_eq!(
            s.server.player_experience(alice.id).map(|xp| xp.0),
            Some(expected),
            "keep_level {keep_level}"
        );
    }
}

#[test]
fn an_entity_death_handler_also_decides_the_death_of_a_player() {
    let mut s = scene_with(&["Alice"], vec![deciding(1, EventKind::EntityDeath)]);
    let alice = s.players[0].clone();
    give(&mut s, "give Alice diamond 5");
    kill(&mut s, &alice);
    let asked = last_asked(&s.plugin, EventKind::PlayerDeath);
    rule_on(
        &mut s.server,
        1,
        asked.id,
        false,
        Some(EventPatch::EntityDeath(EntityDeathPatch {
            dropped_exp: None,
            drops: Some(vec![
                ItemSpec::of("minecraft:diamond", 5),
                ItemSpec::of("minecraft:stick", 2),
            ]),
        })),
    );
    assert_eq!(s.server.items_on_ground(), 2);
}

#[test]
fn a_respawn_waits_behind_the_death_of_its_player() {
    let mut s = scene_with(
        &["Alice"],
        vec![
            deciding(1, EventKind::PlayerDeath),
            observe(2, EventKind::PlayerRespawn),
        ],
    );
    let alice = s.players[0].clone();
    kill(&mut s, &alice);
    send_input(&mut s.server, &alice, PlayerInput::Respawn);
    assert_eq!(s.server.player_is_dead(alice.id), Some(true));
    assert!(settled_for(&s.plugin, EventKind::PlayerRespawn).is_empty());
    let (event, _) = death_of(&s);
    rule_on(&mut s.server, 1, event, false, None);
    assert_eq!(s.server.player_is_dead(alice.id), Some(false));
    assert_eq!(settled_for(&s.plugin, EventKind::PlayerRespawn).len(), 1);
}

#[test]
fn moves_sent_while_a_changed_destination_waits_are_dropped_when_the_teleport_applies() {
    let mut s = scene_with(
        &["Alice"],
        vec![decide(1), deciding(2, EventKind::PlayerTeleport)],
    );
    let alice = s.players[0].clone();
    let step = |s: &mut Scene, x: f64| {
        send_input(
            &mut s.server,
            &alice,
            PlayerInput::Move {
                position: Some(at(x, 0.5)),
                rotation: None,
                on_ground: true,
            },
        );
    };
    step(&mut s, 1.5);
    let moved = last_asked(&s.plugin, EventKind::PlayerMove);
    rule_on(
        &mut s.server,
        1,
        moved.id,
        false,
        Some(EventPatch::PlayerMove(PlayerMovePatch {
            to: Some(location(at(8.5, 0.5))),
        })),
    );
    step(&mut s, 2.5);
    let later = last_asked(&s.plugin, EventKind::PlayerMove);
    assert_ne!(later.id, moved.id);
    let teleport = last_asked(&s.plugin, EventKind::PlayerTeleport);
    judge(&mut s.server, 1, teleport.id, 2, false, None);
    rule_on(&mut s.server, 1, later.id, false, None);
    s.tick(3);
    assert_eq!(s.server.player_location(alice.id), Some(at(8.5, 0.5)));
}
