mod common;

use common::*;
use rspigot_core::events::{Inbound, PlayerInput};
use rspigot_core::plugins::events::{
    PlayerExpChange, PlayerExpChangePatch, PlayerExpCooldownChange, PlayerExpCooldownChangePatch,
    PlayerExpCooldownChangeReason, PlayerLevelChange,
};
use rspigot_core::plugins::{EventKind, EventPatch, EventPayload, Phase, PlayerOrder};

fn orb(s: &mut Scene, value: i32) {
    s.server.handle(Inbound::Console(format!(
        "summon experience_orb 0.5 64 0.5 {{Value:{value}s}}"
    )));
    s.server.deliver();
}

fn level(s: &Scene, player: &Player) -> (i32, i32) {
    s.server
        .player_experience(player.id)
        .map(|(level, _, total)| (level, total))
        .expect("the player is online")
}

fn watch() -> Vec<rspigot_core::plugins::HandlerSpec> {
    vec![
        observe(1, EventKind::PlayerExpCooldownChange),
        observe(2, EventKind::PlayerExpChange),
        observe(3, EventKind::PlayerLevelChange),
    ]
}

#[test]
fn an_orb_fires_the_cooldown_then_the_change_then_the_level_at_the_next_sync() {
    let mut s = scene_with(&["Alice"], watch());
    let alice = s.players[0].clone();
    s.tick(1);
    orb(&mut s, 7);
    s.tick(3);
    assert_eq!(level(&s, &alice), (1, 7));
    assert_eq!(
        fired_kinds(&s.plugin),
        [
            (Phase::Settled, EventKind::PlayerExpCooldownChange),
            (Phase::Settled, EventKind::PlayerExpChange),
            (Phase::Settled, EventKind::PlayerLevelChange),
        ]
    );
    let events = fired(&s.plugin);
    let EventPayload::PlayerExpCooldownChange(PlayerExpCooldownChange {
        new_cooldown,
        reason,
        ..
    }) = &events[0].payload
    else {
        panic!("expected the cooldown");
    };
    assert_eq!(
        (*new_cooldown, *reason),
        (2, PlayerExpCooldownChangeReason::PickupOrb)
    );
    let EventPayload::PlayerExpChange(PlayerExpChange { amount, .. }) = &events[1].payload else {
        panic!("expected the change");
    };
    assert_eq!(*amount, 7);
    let EventPayload::PlayerLevelChange(PlayerLevelChange {
        old_level,
        new_level,
        ..
    }) = &events[2].payload
    else {
        panic!("expected the level");
    };
    assert_eq!((*old_level, *new_level), (0, 1));
}

#[test]
fn a_changed_amount_is_what_the_player_gets_and_the_orb_waits_meanwhile() {
    let mut s = scene_with(&["Alice"], vec![deciding(1, EventKind::PlayerExpChange)]);
    let alice = s.players[0].clone();
    orb(&mut s, 7);
    s.tick(1);
    let asked = asked_for(&s.plugin, EventKind::PlayerExpChange);
    assert_eq!(asked.len(), 1);
    assert_eq!(level(&s, &alice), (0, 0));
    assert_eq!(s.server.experience_orbs().len(), 1);
    rule_on(
        &mut s.server,
        1,
        asked[0].id,
        false,
        Some(EventPatch::PlayerExpChange(PlayerExpChangePatch {
            amount: Some(30),
        })),
    );
    assert_eq!(level(&s, &alice).1, 30);
    assert!(s.server.experience_orbs().is_empty());
}

#[test]
fn an_orb_that_vanished_while_parked_gives_nothing() {
    let mut s = scene_with(&["Alice"], vec![deciding(1, EventKind::PlayerExpChange)]);
    let alice = s.players[0].clone();
    orb(&mut s, 7);
    s.tick(2);
    let asked = last_asked(&s.plugin, EventKind::PlayerExpChange);
    for entity in s.server.experience_orbs() {
        s.act(rspigot_core::plugins::Order::Entity {
            entity: entity.id,
            order: rspigot_core::plugins::EntityOrder::Remove,
        });
    }
    rule_on(&mut s.server, 1, asked.id, false, None);
    assert_eq!(level(&s, &alice), (0, 0));
}

#[test]
fn a_changed_cooldown_keeps_the_next_orb_away() {
    let mut s = scene_with(
        &["Alice"],
        vec![deciding(1, EventKind::PlayerExpCooldownChange)],
    );
    let alice = s.players[0].clone();
    orb(&mut s, 3);
    s.tick(1);
    let asked = last_asked(&s.plugin, EventKind::PlayerExpCooldownChange);
    rule_on(
        &mut s.server,
        1,
        asked.id,
        false,
        Some(EventPatch::PlayerExpCooldownChange(
            PlayerExpCooldownChangePatch {
                new_cooldown: Some(40),
            },
        )),
    );
    assert_eq!(level(&s, &alice).1, 3);
    orb(&mut s, 3);
    s.tick(20);
    assert_eq!(level(&s, &alice).1, 3);
    assert_eq!(
        asked_for(&s.plugin, EventKind::PlayerExpCooldownChange).len(),
        1
    );
    s.tick(25);
    assert_eq!(
        asked_for(&s.plugin, EventKind::PlayerExpCooldownChange).len(),
        2
    );
}

#[test]
fn a_level_change_fires_once_per_sync_from_any_cause_but_not_at_respawn() {
    let mut s = scene_with(&["Alice"], vec![observe(1, EventKind::PlayerLevelChange)]);
    let alice = s.players[0].clone();
    s.tick(1);
    s.act(to(&alice, PlayerOrder::GiveLevels(3)));
    s.act(to(&alice, PlayerOrder::GiveLevels(2)));
    s.tick(1);
    let heard = settled_for(&s.plugin, EventKind::PlayerLevelChange);
    assert_eq!(heard.len(), 1);
    let EventPayload::PlayerLevelChange(PlayerLevelChange {
        old_level,
        new_level,
        ..
    }) = &heard[0].payload
    else {
        panic!("expected a level change");
    };
    assert_eq!((*old_level, *new_level), (0, 5));
    s.act(to(&alice, PlayerOrder::SetHealth(0.0)));
    send_input(&mut s.server, &alice, PlayerInput::Respawn);
    s.tick(2);
    assert_eq!(
        settled_for(&s.plugin, EventKind::PlayerLevelChange).len(),
        1
    );
}
