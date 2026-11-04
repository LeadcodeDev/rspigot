mod common;

use common::*;
use rspigot_core::config::PluginSettings;
use rspigot_core::events::{Inbound, PlayerInput};
use rspigot_core::plugins::events::{
    EntityCombustPatch, EntityPotionEffectAction, EntityPotionEffectCause, EntityPotionEffectPatch,
};
use rspigot_core::plugins::{
    Answer, BlockOrder, EffectSpec, EntityOrder, EntityView, EventKind, EventPatch, EventPayload,
    HandlerSpec, Order, Phase, PlayerOrder, PlayerState, Question,
};
use rspigot_domain::{BlockPos, Vec3};

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

fn spawn(s: &mut Scene, kind: &str, position: Vec3) -> EntityView {
    let Answer::Spawned(view) = s.ask(Question::Spawn {
        kind: kind.into(),
        position,
    }) else {
        panic!("expected the spawned {kind}");
    };
    view
}

fn fire_ticks(s: &mut Scene, entity: &EntityView) -> i32 {
    match s.ask(Question::Entity(entity.id)) {
        Answer::Entity(details) => details.fire_ticks,
        _ => panic!("expected {entity:?}"),
    }
}

fn state(s: &mut Scene) -> PlayerState {
    let Answer::Player(state) = s.ask(Question::Player(s.players[0].id)) else {
        panic!("expected Alice");
    };
    state
}

fn effect(key: &str, duration: i32, amplifier: i32) -> EffectSpec {
    EffectSpec {
        key: key.into(),
        duration,
        amplifier,
        ambient: false,
        particles: true,
        icon: true,
    }
}

fn potions(s: &Scene) -> Vec<(EntityPotionEffectAction, EntityPotionEffectCause, bool)> {
    fired(&s.plugin)
        .into_iter()
        .filter_map(|event| match event.payload {
            EventPayload::EntityPotionEffect(potion) => {
                Some((potion.action, potion.cause, potion.overrides))
            }
            _ => None,
        })
        .collect()
}

fn burning_zombie(s: &mut Scene, block: &str) -> (EntityView, BlockPos) {
    s.act(Order::World(rspigot_core::plugins::WorldOrder::SetTime(
        18_000,
    )));
    let at = Vec3::new(4.5, 64.0, 4.5);
    stream_chunk_of(s, at.block());
    let pos = BlockPos::new(4, 64, 4);
    s.act(Order::Block {
        pos,
        order: BlockOrder::Set(block.into()),
    });
    let zombie = spawn(s, "minecraft:zombie", at);
    (zombie, pos)
}

#[test]
fn a_fire_block_combusts_by_block_and_reaches_the_combust_handlers() {
    let mut s = scene_with(
        &["Alice"],
        vec![
            observe(1, EventKind::EntityCombust),
            observe(2, EventKind::EntityCombustByBlock),
        ],
    );
    let (zombie, pos) = burning_zombie(&mut s, "minecraft:fire");
    s.tick(3);
    let settled = settled_for(&s.plugin, EventKind::EntityCombustByBlock);
    let EventPayload::EntityCombustByBlock(burn) = &settled.first().expect("a combustion").payload
    else {
        panic!("a combustion by block");
    };
    assert_eq!(burn.entity.id, zombie.id);
    assert_eq!(burn.combuster, Some(pos));
    assert_eq!(burn.duration, 8.0);
    assert!(fire_ticks(&mut s, &zombie) > 0);
}

#[test]
fn lava_combusts_once_while_asked_and_a_patched_duration_sets_the_fire() {
    let mut s = waiting_scene(vec![deciding(1, EventKind::EntityCombust)]);
    let (zombie, pos) = burning_zombie(&mut s, "minecraft:lava");
    s.tick(5);
    let asked = asked_for(&s.plugin, EventKind::EntityCombustByBlock);
    assert_eq!(asked.len(), 1);
    let EventPayload::EntityCombustByBlock(burn) = &asked[0].payload else {
        panic!("a combustion by block");
    };
    assert_eq!(burn.combuster, Some(pos));
    assert_eq!(burn.duration, 15.0);
    assert_eq!(fire_ticks(&mut s, &zombie), 0);
    s.act(Order::Block {
        pos,
        order: BlockOrder::Set("minecraft:air".into()),
    });
    rule_on(
        &mut s.server,
        1,
        asked[0].id,
        false,
        Some(EventPatch::EntityCombust(EntityCombustPatch {
            duration: Some(2.0),
        })),
    );
    assert_eq!(fire_ticks(&mut s, &zombie), 40);
    assert!(s.warnings().is_empty());
}

#[test]
fn a_cancelled_combustion_leaves_the_entity_unburnt() {
    let mut s = waiting_scene(vec![deciding(1, EventKind::EntityCombustByBlock)]);
    let (zombie, pos) = burning_zombie(&mut s, "minecraft:fire");
    s.tick(3);
    let asked = last_asked(&s.plugin, EventKind::EntityCombustByBlock);
    s.act(Order::Block {
        pos,
        order: BlockOrder::Set("minecraft:air".into()),
    });
    rule_on(&mut s.server, 1, asked.id, true, None);
    assert!(fire_ticks(&mut s, &zombie) <= 0);
}

#[test]
fn the_effect_command_and_plugin_orders_name_their_cause_and_override() {
    let mut s = scene_with(&["Alice"], vec![observe(1, EventKind::EntityPotionEffect)]);
    console(&mut s, "effect give Alice minecraft:speed 30 0");
    console(&mut s, "effect give Alice minecraft:speed 60 1");
    console(&mut s, "effect give Alice minecraft:speed 10 0");
    let alice = s.players[0].clone();
    s.act(to(
        &alice,
        PlayerOrder::AddEffect(effect("minecraft:haste", 100, 0)),
    ));
    s.act(to(
        &alice,
        PlayerOrder::RemoveEffect("minecraft:haste".into()),
    ));
    console(&mut s, "effect clear Alice");
    assert_eq!(
        potions(&s),
        [
            (
                EntityPotionEffectAction::Added,
                EntityPotionEffectCause::Command,
                true
            ),
            (
                EntityPotionEffectAction::Changed,
                EntityPotionEffectCause::Command,
                true
            ),
            (
                EntityPotionEffectAction::Changed,
                EntityPotionEffectCause::Command,
                false
            ),
            (
                EntityPotionEffectAction::Added,
                EntityPotionEffectCause::Plugin,
                true
            ),
            (
                EntityPotionEffectAction::Removed,
                EntityPotionEffectCause::Plugin,
                true
            ),
            (
                EntityPotionEffectAction::Cleared,
                EntityPotionEffectCause::Command,
                true
            ),
        ]
    );
    assert!(state(&mut s).effects.is_empty());
}

#[test]
fn a_cancelled_or_unoverridden_effect_change_keeps_the_active_effect() {
    let mut s = waiting_scene(vec![deciding(1, EventKind::EntityPotionEffect)]);
    let alice = s.players[0].clone();
    s.act(to(
        &alice,
        PlayerOrder::AddEffect(effect("minecraft:speed", 200, 0)),
    ));
    let asked = last_asked(&s.plugin, EventKind::EntityPotionEffect);
    assert!(state(&mut s).effects.is_empty());
    rule_on(&mut s.server, 1, asked.id, true, None);
    assert!(state(&mut s).effects.is_empty());
    s.act(to(
        &alice,
        PlayerOrder::AddEffect(effect("minecraft:speed", 200, 0)),
    ));
    let asked = last_asked(&s.plugin, EventKind::EntityPotionEffect);
    rule_on(&mut s.server, 1, asked.id, false, None);
    assert_eq!(state(&mut s).effects[0].amplifier, 0);
    s.act(to(
        &alice,
        PlayerOrder::AddEffect(effect("minecraft:speed", 200, 3)),
    ));
    let asked = last_asked(&s.plugin, EventKind::EntityPotionEffect);
    rule_on(
        &mut s.server,
        1,
        asked.id,
        false,
        Some(EventPatch::EntityPotionEffect(EntityPotionEffectPatch {
            overrides: Some(false),
        })),
    );
    assert_eq!(state(&mut s).effects[0].amplifier, 0);
}

#[test]
fn a_cancelled_expiration_keeps_a_spent_effect_and_comes_back_each_tick() {
    let mut s = waiting_scene(vec![deciding(1, EventKind::EntityPotionEffect)]);
    let alice = s.players[0].clone();
    s.act(to(
        &alice,
        PlayerOrder::AddEffect(effect("minecraft:speed", 3, 0)),
    ));
    let added = last_asked(&s.plugin, EventKind::EntityPotionEffect);
    rule_on(&mut s.server, 1, added.id, false, None);
    s.tick(6);
    let expired = asked_for(&s.plugin, EventKind::EntityPotionEffect);
    assert_eq!(expired.len(), 2, "one expiration while it waits");
    let EventPayload::EntityPotionEffect(event) = &expired[1].payload else {
        panic!("an expiration");
    };
    assert_eq!(event.cause, EntityPotionEffectCause::Expiration);
    assert_eq!(event.action, EntityPotionEffectAction::Removed);
    rule_on(&mut s.server, 1, expired[1].id, true, None);
    assert_eq!(state(&mut s).effects.len(), 1);
    assert_eq!(state(&mut s).effects[0].duration, 0);
    s.tick(1);
    let again = last_asked(&s.plugin, EventKind::EntityPotionEffect);
    assert_ne!(again.id, expired[1].id);
    rule_on(&mut s.server, 1, again.id, false, None);
    assert!(state(&mut s).effects.is_empty());
}

#[test]
fn a_respawn_clears_the_effects_with_the_death_cause() {
    let mut s = scene_with(&["Alice"], vec![observe(1, EventKind::EntityPotionEffect)]);
    let alice = s.players[0].clone();
    s.act(to(
        &alice,
        PlayerOrder::AddEffect(effect("minecraft:speed", 200, 0)),
    ));
    s.act(to(&alice, PlayerOrder::Kill));
    send_input(&mut s.server, &alice, PlayerInput::Respawn);
    assert_eq!(
        potions(&s).last(),
        Some(&(
            EntityPotionEffectAction::Cleared,
            EntityPotionEffectCause::Death,
            true
        ))
    );
    assert!(state(&mut s).effects.is_empty());
}

#[test]
fn mob_effects_from_orders_and_their_expiration_fire_too() {
    let mut s = scene_with(&["Alice"], vec![observe(1, EventKind::EntityPotionEffect)]);
    let cow = spawn(&mut s, "minecraft:cow", Vec3::new(2.5, 64.0, 2.5));
    s.act(Order::Entity {
        entity: cow.id,
        order: EntityOrder::AddEffect(effect("minecraft:slowness", 2, 0)),
    });
    s.tick(4);
    assert_eq!(
        potions(&s),
        [
            (
                EntityPotionEffectAction::Added,
                EntityPotionEffectCause::Plugin,
                true
            ),
            (
                EntityPotionEffectAction::Removed,
                EntityPotionEffectCause::Expiration,
                true
            ),
        ]
    );
}

#[test]
fn daylight_combusts_an_undead_without_a_combuster() {
    let mut s = scene_with(&["Alice"], vec![observe(1, EventKind::EntityCombust)]);
    s.act(Order::World(rspigot_core::plugins::WorldOrder::SetTime(
        6000,
    )));
    let at = Vec3::new(4.5, 64.0, 4.5);
    stream_chunk_of(&mut s, at.block());
    console(&mut s, "summon minecraft:zombie 4.5 64 4.5 {NoAI:1b}");
    for _ in 0..600 {
        s.tick(1);
        if !settled_for(&s.plugin, EventKind::EntityCombust).is_empty() {
            break;
        }
    }
    let settled = settled_for(&s.plugin, EventKind::EntityCombust);
    let EventPayload::EntityCombust(burn) = &settled.first().expect("the sun burns").payload else {
        panic!("a plain combustion");
    };
    assert_eq!(burn.entity_type, "minecraft:zombie");
    assert_eq!(burn.duration, 8.0);
}

#[test]
fn a_shutdown_settles_the_waiting_events_of_an_entity_lane() {
    let mut s = waiting_scene(vec![
        deciding(1, EventKind::EntityPotionEffect),
        observe(2, EventKind::EntityPotionEffect),
    ]);
    let cow = spawn(&mut s, "minecraft:cow", Vec3::new(2.5, 64.0, 2.5));
    s.act(Order::Entity {
        entity: cow.id,
        order: EntityOrder::AddEffect(effect("minecraft:slowness", 200, 0)),
    });
    assert_eq!(asked_for(&s.plugin, EventKind::EntityPotionEffect).len(), 1);
    assert!(settled_for(&s.plugin, EventKind::EntityPotionEffect).is_empty());
    let (sender, inbound) = crossbeam_channel::unbounded();
    sender.send(Inbound::Shutdown).expect("open");
    let Scene { server, plugin, .. } = s;
    let _ = server.run(&inbound);
    let settled = settled_for(&plugin, EventKind::EntityPotionEffect);
    assert_eq!(settled.len(), 1);
    assert!(!settled[0].cancelled);
}

fn flaming_arrow_at(s: &mut Scene, target: &EntityView) {
    let position = match s.ask(Question::Entity(target.id)) {
        Answer::Entity(details) => details.entity.position,
        _ => panic!("expected {target:?}"),
    };
    console(
        s,
        &format!(
            "summon minecraft:arrow {} {} {} {{Motion:[0.0,0.0,1.0],Fire:200s}}",
            position.x,
            position.y + 0.5,
            position.z - 3.0
        ),
    );
}

fn combust_by_entity(
    s: &mut Scene,
    before: usize,
) -> rspigot_core::plugins::events::EntityCombustByEntity {
    for _ in 0..40 {
        s.tick(1);
        let asked: Vec<Fired> = fired(&s.plugin)
            .into_iter()
            .filter(|event| {
                event.phase == Phase::Decide
                    && matches!(event.payload, EventPayload::EntityCombustByEntity(_))
            })
            .collect();
        if asked.len() > before {
            let EventPayload::EntityCombustByEntity(burn) = &asked[before].payload else {
                unreachable!();
            };
            return burn.clone();
        }
    }
    panic!("no EntityCombustByEntity");
}

#[test]
fn a_flaming_arrow_combusts_its_target_by_entity() {
    let mut s = waiting_scene(vec![
        deciding(1, EventKind::EntityCombust),
        observe(2, EventKind::EntityCombustByEntity),
    ]);
    stream_chunk_of(&mut s, BlockPos::new(4, 64, 8));
    console(&mut s, "summon minecraft:cow 4.5 64 8.5 {NoAI:1b}");
    let id = s.server.entities_of(rspigot_entities::EntityKind::Cow)[0].id;
    let Answer::Entity(details) = s.ask(Question::Entity(id)) else {
        panic!("expected the cow");
    };
    let cow = details.entity;
    flaming_arrow_at(&mut s, &cow);
    let burn = combust_by_entity(&mut s, 0);
    assert_eq!(burn.entity.id, cow.id);
    assert_eq!(burn.entity_type, "minecraft:cow");
    assert_eq!(burn.combuster.kind, "minecraft:arrow");
    assert_eq!(burn.duration, 5.0);
    let asked = last_asked(&s.plugin, EventKind::EntityCombustByEntity);
    rule_on(&mut s.server, 1, asked.id, true, None);
    assert!(fire_ticks(&mut s, &cow) <= 0);
    s.tick(20);
    flaming_arrow_at(&mut s, &cow);
    combust_by_entity(&mut s, 1);
    let asked = last_asked(&s.plugin, EventKind::EntityCombustByEntity);
    rule_on(
        &mut s.server,
        1,
        asked.id,
        false,
        Some(EventPatch::EntityCombust(EntityCombustPatch {
            duration: Some(2.0),
        })),
    );
    assert_eq!(fire_ticks(&mut s, &cow), 40);
    let settled = settled_for(&s.plugin, EventKind::EntityCombustByEntity);
    assert_eq!(settled.len(), 2);
    assert!(settled[0].cancelled);
    assert!(!settled[1].cancelled);
    assert!(s.warnings().is_empty());
}

#[test]
fn a_flaming_arrow_is_asked_while_a_sun_ignition_of_its_target_waits() {
    let mut s = waiting_scene(vec![deciding(1, EventKind::EntityCombust)]);
    s.act(Order::World(rspigot_core::plugins::WorldOrder::SetTime(
        6000,
    )));
    stream_chunk_of(&mut s, BlockPos::new(4, 64, 8));
    console(&mut s, "summon minecraft:zombie 4.5 64 8.5 {NoAI:1b}");
    let id = s.server.entities_of(rspigot_entities::EntityKind::Zombie)[0].id;
    for _ in 0..600 {
        s.tick(1);
        if !asked_for(&s.plugin, EventKind::EntityCombust).is_empty() {
            break;
        }
    }
    assert_eq!(asked_for(&s.plugin, EventKind::EntityCombust).len(), 1);
    let Answer::Entity(details) = s.ask(Question::Entity(id)) else {
        panic!("expected the zombie");
    };
    flaming_arrow_at(&mut s, &details.entity);
    let burn = combust_by_entity(&mut s, 0);
    assert_eq!(burn.entity.id, id);
    assert_eq!(asked_for(&s.plugin, EventKind::EntityCombust).len(), 1);
}

#[test]
fn a_bounced_flaming_arrow_leaves_its_target_unburnt_once_the_ignition_settles() {
    let mut s = waiting_scene(vec![deciding(1, EventKind::EntityCombust)]);
    stream_chunk_of(&mut s, BlockPos::new(4, 64, 8));
    console(
        &mut s,
        "summon minecraft:cow 4.5 64 8.5 {NoAI:1b,Invulnerable:1b}",
    );
    let id = s.server.entities_of(rspigot_entities::EntityKind::Cow)[0].id;
    let Answer::Entity(details) = s.ask(Question::Entity(id)) else {
        panic!("expected the cow");
    };
    let cow = details.entity;
    flaming_arrow_at(&mut s, &cow);
    combust_by_entity(&mut s, 0);
    let asked = last_asked(&s.plugin, EventKind::EntityCombustByEntity);
    rule_on(&mut s.server, 1, asked.id, false, None);
    assert!(fire_ticks(&mut s, &cow) <= 0);
}
