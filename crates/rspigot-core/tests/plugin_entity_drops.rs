mod common;

use common::*;
use rspigot_core::config::PluginSettings;
use rspigot_core::events::{ClientEvent, Hand, Inbound, PlayerInput};
use rspigot_core::plugins::events::EntityDropItem;
use rspigot_core::plugins::{BlockOrder, EventKind, EventPayload, HandlerSpec, Order};
use rspigot_domain::{BlockPos, EntityId, Vec3};
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

fn deciding_drops() -> Scene {
    let mut s = waiting_scene(vec![
        deciding(1, EventKind::EntityDropItem),
        observe(2, EventKind::EntityDropItem),
    ]);
    stream_chunk_of(&mut s, BlockPos::new(8, 64, 8));
    s
}

fn console(s: &mut Scene, line: &str) {
    s.server.handle(Inbound::Console(line.to_owned()));
    s.server.deliver();
}

fn summoned(s: &mut Scene, line: &str, kind: Kind) -> EntityId {
    let before: Vec<EntityId> = s.server.entities_of(kind).iter().map(|e| e.id).collect();
    console(s, line);
    s.server
        .entities_of(kind)
        .iter()
        .map(|e| e.id)
        .find(|id| !before.contains(id))
        .expect("the summoned entity")
}

fn asked_drops(s: &Scene) -> Vec<(u64, EntityDropItem)> {
    asked_for(&s.plugin, EventKind::EntityDropItem)
        .into_iter()
        .filter_map(|event| match event.payload {
            EventPayload::EntityDropItem(drop) => Some((event.id, drop)),
            _ => None,
        })
        .collect()
}

fn tick_until_dropped(s: &mut Scene, before: usize) -> Vec<(u64, EntityDropItem)> {
    for _ in 0..200 {
        if asked_drops(s).len() > before {
            return asked_drops(s).split_off(before);
        }
        s.tick(1);
    }
    panic!("no EntityDropItem was asked");
}

fn rule_all(s: &mut Scene, drops: &[(u64, EntityDropItem)], cancel: bool) {
    for (id, _) in drops {
        rule_on(&mut s.server, 1, *id, cancel, None);
    }
}

fn interact(s: &mut Scene, target: EntityId) {
    let alice = s.players[0].clone();
    send_input(
        &mut s.server,
        &alice,
        PlayerInput::Interact {
            target,
            hand: Hand::Main,
            sneaking: false,
            location: Vec3::new(0.0, 0.5, 0.0),
        },
    );
}

fn sounds(s: &Scene) -> Vec<&'static str> {
    s.players[0]
        .link
        .take()
        .into_iter()
        .filter_map(|event| match event {
            ClientEvent::Sound { sound, .. } => Some(sound),
            _ => None,
        })
        .collect()
}

#[test]
fn a_laid_egg_asks_its_drop_and_a_cancel_keeps_the_sound_and_the_timer() {
    let mut s = deciding_drops();
    let chicken = summoned(
        &mut s,
        "summon minecraft:chicken 2.5 64 2.5 {NoAI:1b,EggLayTime:2}",
        Kind::Chicken,
    );
    s.players[0].link.take();
    let drops = tick_until_dropped(&mut s, 0);
    assert_eq!(drops.len(), 1);
    let (_, drop) = &drops[0];
    assert_eq!(drop.entity.id, chicken);
    assert_eq!(drop.entity_type, "minecraft:chicken");
    assert_eq!(drop.item_drop.kind, "minecraft:item");
    s.server.deliver();
    assert!(sounds(&s).contains(&"minecraft:entity.chicken.egg"));
    rule_all(&mut s, &drops, true);
    assert_eq!(s.server.items_on_ground(), 0);
    s.tick(100);
    assert_eq!(asked_drops(&s).len(), 1, "the egg timer was reset");
    let settled = settled_for(&s.plugin, EventKind::EntityDropItem);
    assert_eq!(settled.len(), 1);
    assert!(settled[0].cancelled);
}

#[test]
fn shorn_wool_asks_its_drop_and_a_cancel_still_shears_the_sheep() {
    let mut s = deciding_drops();
    console(&mut s, "give Alice shears 1");
    let sheep = summoned(
        &mut s,
        "summon minecraft:sheep 1.5 64 0.5 {NoAI:1b}",
        Kind::Sheep,
    );
    interact(&mut s, sheep);
    let drops = tick_until_dropped(&mut s, 0);
    assert!(!drops.is_empty());
    assert!(drops.iter().all(|(_, drop)| drop.entity.id == sheep));
    rule_all(&mut s, &drops, true);
    assert_eq!(s.server.items_on_ground(), 0);
    interact(&mut s, sheep);
    s.tick(2);
    assert_eq!(asked_drops(&s).len(), drops.len(), "the sheep is shorn");
}

#[test]
fn a_shorn_saddle_asks_its_drop_and_an_allowed_one_lands() {
    let mut s = deciding_drops();
    let pig = summoned(
        &mut s,
        "summon minecraft:pig 1.5 64 0.5 {NoAI:1b}",
        Kind::Pig,
    );
    console(&mut s, "give Alice saddle 1");
    interact(&mut s, pig);
    s.tick(1);
    console(&mut s, "clear Alice");
    console(&mut s, "give Alice shears 1");
    interact(&mut s, pig);
    let drops = tick_until_dropped(&mut s, 0);
    assert_eq!(drops.len(), 1);
    assert_eq!(drops[0].1.entity.id, pig);
    rule_all(&mut s, &drops, false);
    let saddle = rspigot_items::ItemKind::Saddle.id();
    assert!(
        s.server
            .ground_items()
            .iter()
            .any(|(_, stack)| stack.item == saddle)
    );
}

#[test]
fn an_arrow_stopped_by_a_target_it_cannot_hurt_asks_its_drop() {
    let mut s = deciding_drops();
    summoned(
        &mut s,
        "summon minecraft:cow 8.5 64 2.5 {NoAI:1b,Invulnerable:1b}",
        Kind::Cow,
    );
    let arrow = summoned(
        &mut s,
        "summon minecraft:arrow 8.5 64.5 2.049 {NoGravity:1b,pickup:1b,Motion:[0.0,0.0,0.002]}",
        Kind::Arrow,
    );
    let drops = tick_until_dropped(&mut s, 0);
    assert_eq!(drops.len(), 1);
    assert_eq!(drops[0].1.entity.id, arrow);
    assert_eq!(drops[0].1.entity_type, "minecraft:arrow");
    rule_all(&mut s, &drops, true);
    assert_eq!(s.server.items_on_ground(), 0);
}

#[test]
fn a_falling_block_that_cannot_land_asks_its_drop_and_is_removed_on_a_cancel() {
    let mut s = deciding_drops();
    let torch = BlockPos::new(5, 64, 5);
    s.act(Order::Block {
        pos: torch,
        order: BlockOrder::Set("minecraft:torch".into()),
    });
    let falling = summoned(
        &mut s,
        "summon minecraft:falling_block 5.5 66 5.5 {BlockState:{Name:\"minecraft:sand\"}}",
        Kind::FallingBlock,
    );
    let drops = tick_until_dropped(&mut s, 0);
    assert_eq!(drops.len(), 1);
    assert_eq!(drops[0].1.entity.id, falling);
    assert_eq!(drops[0].1.entity_type, "minecraft:falling_block");
    rule_all(&mut s, &drops, true);
    assert_eq!(s.server.items_on_ground(), 0);
    assert_eq!(s.server.falling_blocks(), 0);
}
