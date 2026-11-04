mod common;

use common::*;
use rspigot_core::config::PluginSettings;
use rspigot_core::events::{ClientEvent, Inbound};
use rspigot_core::plugins::events::{
    EntityExplodePatch, EntityRemoveCause, EntityTeleportPatch, ExplosionPrimePatch,
};
use rspigot_core::plugins::{
    Answer, EntityOrder, EntityView, EventKind, EventPatch, EventPayload, HandlerSpec, Location,
    Order, Question,
};
use rspigot_domain::{BlockPos, EntityId, Rotation, Vec3};
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

fn creeper(s: &mut Scene) -> EntityId {
    let at = BlockPos::new(8, 64, 8);
    stream_chunk_of(s, at);
    console(
        s,
        "summon minecraft:creeper 8.5 64 8.5 {NoAI:1b,ignited:1b,Fuse:2s}",
    );
    let found = s.server.entities_of(Kind::Creeper);
    assert_eq!(found.len(), 1);
    found[0].id
}

fn block(s: &mut Scene, pos: BlockPos) -> String {
    match s.ask(Question::Block(pos)) {
        Answer::Block(state) => state.kind,
        _ => "minecraft:air".to_owned(),
    }
}

fn exists(s: &mut Scene, id: EntityId) -> bool {
    !matches!(s.ask(Question::Entity(id)), Answer::NotFound)
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

fn position(s: &mut Scene, id: EntityId) -> Vec3 {
    match s.ask(Question::Entity(id)) {
        Answer::Entity(details) => details.entity.position,
        _ => panic!("expected the entity {id:?}"),
    }
}

#[test]
fn a_creeper_primes_explodes_then_leaves_with_the_explode_cause() {
    let mut s = scene_with(
        &["Alice"],
        vec![
            observe(1, EventKind::ExplosionPrime),
            observe(2, EventKind::EntityExplode),
            observe(3, EventKind::EntityRemove),
        ],
    );
    let id = creeper(&mut s);
    s.tick(5);
    let kinds: Vec<EventKind> = fired(&s.plugin)
        .into_iter()
        .map(|event| event.payload.kind())
        .collect();
    assert_eq!(
        kinds,
        [
            EventKind::ExplosionPrime,
            EventKind::EntityExplode,
            EventKind::EntityRemove
        ]
    );
    let EventPayload::ExplosionPrime(prime) =
        &settled_for(&s.plugin, EventKind::ExplosionPrime)[0].payload
    else {
        panic!("an explosion prime");
    };
    assert_eq!(prime.entity.id, id);
    assert_eq!(prime.radius, 3.0);
    assert!(!prime.fire);
    let EventPayload::EntityExplode(explode) =
        &settled_for(&s.plugin, EventKind::EntityExplode)[0].payload
    else {
        panic!("an entity explode");
    };
    assert_eq!(explode.entity.id, id);
    assert!(!explode.block_list.is_empty());
    let EventPayload::EntityRemove(removed) =
        &settled_for(&s.plugin, EventKind::EntityRemove)[0].payload
    else {
        panic!("a removal");
    };
    assert_eq!(removed.cause, EntityRemoveCause::Explode);
    assert!(!exists(&mut s, id));
}

#[test]
fn a_cancelled_prime_keeps_the_creeper_which_primes_again() {
    let mut s = waiting_scene(vec![deciding(1, EventKind::ExplosionPrime)]);
    let id = creeper(&mut s);
    s.tick(5);
    let asked = asked_for(&s.plugin, EventKind::ExplosionPrime);
    assert_eq!(asked.len(), 1);
    rule_on(&mut s.server, 1, asked[0].id, true, None);
    assert!(exists(&mut s, id));
    assert_eq!(s.server.mob_damage(id).map(|state| state.dead), Some(false));
    s.tick(5);
    assert_eq!(asked_for(&s.plugin, EventKind::ExplosionPrime).len(), 2);
}

#[test]
fn a_primed_fire_burns_the_crater() {
    let mut s = waiting_scene(vec![deciding(1, EventKind::ExplosionPrime)]);
    creeper(&mut s);
    s.tick(5);
    let asked = last_asked(&s.plugin, EventKind::ExplosionPrime);
    rule_on(
        &mut s.server,
        1,
        asked.id,
        false,
        Some(EventPatch::ExplosionPrime(ExplosionPrimePatch {
            fire: Some(true),
            radius: Some(12.0),
        })),
    );
    let mut fires = 0;
    for x in -6..=22 {
        for z in -6..=22 {
            for y in 59..=64 {
                if block(&mut s, BlockPos::new(x, y, z)) == "minecraft:fire" {
                    fires += 1;
                }
            }
        }
    }
    assert!(fires > 0);
}

#[test]
fn a_cancelled_entity_explosion_breaks_nothing_but_still_hurts_and_removes_the_creeper() {
    let mut s = waiting_scene(vec![
        deciding(1, EventKind::EntityExplode),
        observe(2, EventKind::EntityDamage),
    ]);
    let alice = s.players[0].clone();
    let id = creeper(&mut s);
    spawn(&mut s, "minecraft:cow", Vec3::new(9.5, 64.0, 8.5));
    s.tick(5);
    let asked = last_asked(&s.plugin, EventKind::EntityExplode);
    assert!(!settled_for(&s.plugin, EventKind::EntityDamageByEntity).is_empty());
    let floor = BlockPos::new(8, 63, 8);
    assert_ne!(block(&mut s, floor), "minecraft:air");
    assert!(exists(&mut s, id), "the creeper waits, dead, in the world");
    alice.link.take();
    rule_on(&mut s.server, 1, asked.id, true, None);
    assert_ne!(block(&mut s, floor), "minecraft:air");
    assert!(!exists(&mut s, id));
    assert!(
        !alice
            .link
            .take()
            .iter()
            .any(|event| matches!(event, ClientEvent::Explode { .. }))
    );
}

#[test]
fn a_patched_block_list_is_what_the_explosion_breaks() {
    let mut s = waiting_scene(vec![deciding(1, EventKind::EntityExplode)]);
    creeper(&mut s);
    s.tick(5);
    let asked = last_asked(&s.plugin, EventKind::EntityExplode);
    let EventPayload::EntityExplode(blast) = &asked.payload else {
        panic!("an entity explosion");
    };
    let far = BlockPos::new(3, 60, 3);
    rule_on(
        &mut s.server,
        1,
        asked.id,
        false,
        Some(EventPatch::EntityExplode(EntityExplodePatch {
            block_list: replacing(&blast.block_list, vec![far]),
            explosion_yield: Some(0.0),
        })),
    );
    assert_eq!(block(&mut s, far), "minecraft:air");
    assert_ne!(block(&mut s, BlockPos::new(8, 63, 8)), "minecraft:air");
    assert_eq!(s.server.items_on_ground(), 0);
}

#[test]
fn a_tp_of_an_entity_asks_twice_and_only_the_second_destination_counts() {
    let mut s = waiting_scene(vec![deciding(1, EventKind::EntityTeleport)]);
    let cow = spawn(&mut s, "minecraft:cow", Vec3::new(2.5, 64.0, 2.5));
    console(&mut s, "tp @e[type=cow,limit=1] 6.5 64 6.5");
    let first = last_asked(&s.plugin, EventKind::EntityTeleport);
    let EventPayload::EntityTeleport(teleport) = &first.payload else {
        panic!("an entity teleport");
    };
    assert_eq!(teleport.entity.id, cow.id);
    assert_eq!(teleport.from.position, Vec3::new(2.5, 64.0, 2.5));
    let patch = |x: f64| {
        Some(EventPatch::EntityTeleport(EntityTeleportPatch {
            to: Some(Some(Location {
                world: "minecraft:overworld".into(),
                position: Vec3::new(x, 64.0, 0.5),
                rotation: Rotation::default(),
            })),
        }))
    };
    rule_on(&mut s.server, 1, first.id, false, patch(9.5));
    let second = last_asked(&s.plugin, EventKind::EntityTeleport);
    assert_ne!(second.id, first.id);
    let EventPayload::EntityTeleport(teleport) = &second.payload else {
        panic!("an entity teleport");
    };
    assert_eq!(
        teleport.to.as_ref().map(|to| to.position),
        Some(Vec3::new(6.5, 64.0, 6.5))
    );
    assert_eq!(position(&mut s, cow.id), Vec3::new(2.5, 64.0, 2.5));
    rule_on(&mut s.server, 1, second.id, false, patch(4.5));
    assert_eq!(position(&mut s, cow.id), Vec3::new(4.5, 64.0, 0.5));
    s.act(Order::Entity {
        entity: cow.id,
        order: EntityOrder::Teleport(Vec3::new(1.5, 64.0, 1.5)),
    });
    let order = last_asked(&s.plugin, EventKind::EntityTeleport);
    rule_on(&mut s.server, 1, order.id, true, None);
    assert_eq!(position(&mut s, cow.id), Vec3::new(4.5, 64.0, 0.5));
    assert_eq!(asked_for(&s.plugin, EventKind::EntityTeleport).len(), 3);
}
