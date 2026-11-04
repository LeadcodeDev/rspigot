mod common;

use common::*;
use rspigot_core::config::PluginSettings;
use rspigot_core::events::Inbound;
use rspigot_core::plugins::events::{EntityTargetLivingEntityPatch, EntityTargetReason};
use rspigot_core::plugins::{Answer, EventKind, EventPatch, EventPayload, HandlerSpec, Question};
use rspigot_domain::{BlockPos, EntityId};
use rspigot_entities::EntityKind as Kind;

fn waiting_scene(handlers: Vec<HandlerSpec>) -> Scene {
    let mut server = server(PluginSettings {
        decision_ticks: 1_000,
        ..PluginSettings::default()
    });
    let plugin = connect_with(&mut server, 1, "tester", handlers);
    let alice = join(&mut server, "Alice");
    server.deliver();
    alice.link.take();
    Scene {
        server,
        plugin,
        players: vec![alice],
    }
}

fn console(s: &mut Scene, line: &str) {
    s.server.handle(Inbound::Console(line.to_owned()));
    s.server.deliver();
}

fn alice_entity(s: &mut Scene) -> EntityId {
    let Answer::Player(state) = s.ask(Question::Player(s.players[0].id)) else {
        panic!("expected Alice");
    };
    EntityId(state.player.entity_id)
}

fn summon(s: &mut Scene, kind: Kind, line: &str) -> EntityId {
    stream_chunk_of(s, BlockPos::new(4, 64, 4));
    console(s, line);
    s.server.entities_of(kind)[0].id
}

fn targets(s: &Scene) -> Vec<(EntityTargetReason, Option<EntityId>)> {
    fired(&s.plugin)
        .into_iter()
        .filter_map(|event| match event.payload {
            EventPayload::EntityTargetLivingEntity(target) => {
                Some((target.reason, target.target.map(|view| view.id)))
            }
            _ => None,
        })
        .collect()
}

fn tick_until_asked(s: &mut Scene, kind: EventKind) {
    for _ in 0..600 {
        s.tick(1);
        if !asked_for(&s.plugin, kind).is_empty() {
            return;
        }
    }
    panic!("no {kind:?} was asked");
}

#[test]
fn a_zombie_picking_the_closest_player_reaches_entity_target_handlers() {
    let mut s = scene_with(&["Alice"], vec![observe(1, EventKind::EntityTarget)]);
    console(&mut s, "time set 18000");
    let zombie = summon(&mut s, Kind::Zombie, "summon minecraft:zombie 4.5 64 4.5");
    let alice = alice_entity(&mut s);
    for _ in 0..600 {
        s.tick(1);
        if !targets(&s).is_empty() {
            break;
        }
    }
    assert_eq!(
        targets(&s).first(),
        Some(&(EntityTargetReason::ClosestPlayer, Some(alice)))
    );
    assert_eq!(s.server.mob_target(zombie), Some(alice));
}

#[test]
fn a_waiting_target_lands_one_ai_pass_later_and_a_cancel_keeps_the_old_one() {
    let mut s = waiting_scene(vec![deciding(1, EventKind::EntityTargetLivingEntity)]);
    console(&mut s, "time set 18000");
    let zombie = summon(&mut s, Kind::Zombie, "summon minecraft:zombie 4.5 64 4.5");
    let alice = alice_entity(&mut s);
    tick_until_asked(&mut s, EventKind::EntityTargetLivingEntity);
    s.tick(20);
    let asked = asks_of(&s, zombie);
    assert_eq!(asked.len(), 1, "one ask while it waits");
    assert_eq!(s.server.mob_target(zombie), None);
    rule_on(&mut s.server, 1, asked[0].id, true, None);
    assert_eq!(s.server.mob_target(zombie), None);
    tick_until_again(&mut s, zombie, asked[0].id);
    let again = asks_of(&s, zombie).pop().expect("asked again");
    rule_on(&mut s.server, 1, again.id, false, None);
    assert_eq!(s.server.mob_target(zombie), Some(alice));
}

fn asks_of(s: &Scene, mob: EntityId) -> Vec<Fired> {
    asked_for(&s.plugin, EventKind::EntityTargetLivingEntity)
        .into_iter()
        .filter(|event| {
            matches!(&event.payload, EventPayload::EntityTargetLivingEntity(target)
                if target.entity.id == mob)
        })
        .collect()
}

fn tick_until_again(s: &mut Scene, mob: EntityId, first: u64) {
    for _ in 0..600 {
        s.tick(1);
        if asks_of(s, mob)
            .last()
            .is_some_and(|event| event.id != first)
        {
            return;
        }
    }
    panic!("the target was never asked again");
}

#[test]
fn a_patched_target_is_the_one_the_mob_takes() {
    let mut s = waiting_scene(vec![deciding(1, EventKind::EntityTargetLivingEntity)]);
    console(&mut s, "time set 18000");
    let zombie = summon(&mut s, Kind::Zombie, "summon minecraft:zombie 4.5 64 4.5");
    console(&mut s, "summon minecraft:cow 8.5 64 8.5 {NoAI:1b}");
    let cow = s.server.entities_of(Kind::Cow)[0].id;
    for _ in 0..600 {
        s.tick(1);
        if !asks_of(&s, zombie).is_empty() {
            break;
        }
    }
    let asked = asks_of(&s, zombie).pop().expect("the zombie asked");
    let Answer::Entity(details) = s.ask(Question::Entity(cow)) else {
        panic!("the cow");
    };
    rule_on(
        &mut s.server,
        1,
        asked.id,
        false,
        Some(EventPatch::EntityTargetLivingEntity(
            EntityTargetLivingEntityPatch {
                target: Some(Some(details.entity)),
            },
        )),
    );
    assert_eq!(s.server.mob_target(zombie), Some(cow));
}

#[test]
fn a_hurt_zombie_targets_its_attacker_and_alerts_the_others() {
    let mut s = scene_with(
        &["Alice"],
        vec![observe(1, EventKind::EntityTargetLivingEntity)],
    );
    console(&mut s, "time set 18000");
    console(&mut s, "gamemode creative Alice");
    stream_chunk_of(&mut s, BlockPos::new(4, 64, 4));
    console(&mut s, "summon minecraft:zombie 4.5 64 4.5");
    console(&mut s, "summon minecraft:zombie 5.5 64 4.5");
    console(&mut s, "summon minecraft:zombie 6.5 64 6.5");
    let zombies: Vec<EntityId> = s
        .server
        .entities_of(Kind::Zombie)
        .into_iter()
        .map(|zombie| zombie.id)
        .collect();
    let (attacker, victim) = (zombies[0], zombies[1]);
    s.tick(5);
    assert!(
        s.server
            .do_hurt_target(attacker, rspigot_core::commands::Target::Entity(victim))
    );
    for _ in 0..100 {
        s.tick(1);
        let heard = targets(&s);
        if heard.contains(&(EntityTargetReason::TargetAttackedEntity, Some(attacker)))
            && heard.contains(&(
                EntityTargetReason::TargetAttackedNearbyEntity,
                Some(attacker),
            ))
        {
            return;
        }
    }
    panic!("{:?}", targets(&s));
}

#[test]
fn a_tempted_cow_waits_for_its_verdict_before_following() {
    let mut s = waiting_scene(vec![deciding(1, EventKind::EntityTargetLivingEntity)]);
    console(&mut s, "give Alice minecraft:wheat");
    summon(&mut s, Kind::Cow, "summon minecraft:cow 3.5 64 3.5");
    tick_until_asked(&mut s, EventKind::EntityTargetLivingEntity);
    let asked = last_asked(&s.plugin, EventKind::EntityTargetLivingEntity);
    let EventPayload::EntityTargetLivingEntity(target) = &asked.payload else {
        panic!("a target");
    };
    assert_eq!(target.reason, EntityTargetReason::Tempt);
    assert_eq!(target.entity_type, "minecraft:cow");
    let alice = alice_entity(&mut s);
    assert_eq!(target.target.as_ref().map(|view| view.id), Some(alice));
    s.tick(10);
    assert_eq!(
        asked_for(&s.plugin, EventKind::EntityTargetLivingEntity).len(),
        1
    );
}

#[test]
fn an_orb_choosing_a_player_fires_closest_player() {
    let mut s = scene_with(
        &["Alice"],
        vec![observe(1, EventKind::EntityTargetLivingEntity)],
    );
    s.server.give_experience(s.players[0].id, 0);
    let Answer::Spawned(_) = s.ask(Question::Spawn {
        kind: "minecraft:experience_orb".into(),
        position: rspigot_domain::Vec3::new(2.5, 64.0, 0.5),
    }) else {
        panic!("an orb");
    };
    s.tick(3);
    let alice = alice_entity(&mut s);
    assert!(
        targets(&s).contains(&(EntityTargetReason::ClosestPlayer, Some(alice))),
        "{:?}",
        targets(&s)
    );
}

#[test]
fn an_orb_whose_player_left_forgets_them_once() {
    let mut server = server(PluginSettings {
        decision_ticks: 1_000,
        ..PluginSettings::default()
    });
    let plugin = connect_with(
        &mut server,
        1,
        "tester",
        vec![deciding(1, EventKind::EntityTargetLivingEntity)],
    );
    let alice = join(&mut server, "Alice");
    let bob = join(&mut server, "Bob");
    let mut s = Scene {
        server,
        plugin,
        players: vec![alice.clone(), bob],
    };
    console(&mut s, "tp Bob 40.5 64 0.5");
    s.server.give_experience(alice.id, 0);
    let Answer::Spawned(orb) = s.ask(Question::Spawn {
        kind: "minecraft:experience_orb".into(),
        position: rspigot_domain::Vec3::new(2.5, 64.0, 0.5),
    }) else {
        panic!("an orb");
    };
    let mut answered = Vec::new();
    let mut answer_all = |s: &mut Scene| {
        let mut asked = 0;
        for event in asked_for(&s.plugin, EventKind::EntityTargetLivingEntity) {
            let EventPayload::EntityTargetLivingEntity(target) = &event.payload else {
                continue;
            };
            if target.entity.id != orb.id || answered.contains(&event.id) {
                continue;
            }
            answered.push(event.id);
            asked += 1;
            rule_on(&mut s.server, 1, event.id, false, None);
        }
        asked
    };
    tick_until_asked(&mut s, EventKind::EntityTargetLivingEntity);
    assert_eq!(answer_all(&mut s), 1);
    s.tick(2);
    answer_all(&mut s);
    s.server.handle(leave_of(&alice));
    s.server.deliver();
    let mut after = 0;
    for _ in 0..10 {
        s.tick(1);
        after += answer_all(&mut s);
    }
    assert_eq!(after, 1);
}
