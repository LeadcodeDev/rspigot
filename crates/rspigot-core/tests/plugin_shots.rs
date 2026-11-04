mod common;

use common::*;
use rspigot_core::config::PluginSettings;
use rspigot_core::events::{ClientEvent, DigAction, Direction, Hand, Inbound, PlayerInput};
use rspigot_core::plugins::events::{EntityShootBowPatch, EquipmentSlot};
use rspigot_core::plugins::{
    Answer, BlockOrder, EntityView, EventKind, EventPatch, EventPayload, HandlerSpec, Order,
    Question,
};
use rspigot_domain::{BlockPos, EntityId, Rotation};
use rspigot_entities::EntityKind as Kind;
use uuid::Uuid;

const AIM: Rotation = Rotation::new(0.0, 0.0);

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

fn shoot(s: &mut Scene) {
    let aim = AIM;
    let alice = s.players[0].clone();
    console(s, "give Alice minecraft:bow");
    console(s, "give Alice minecraft:arrow 2");
    send_input(
        &mut s.server,
        &alice,
        PlayerInput::UseItem {
            hand: Hand::Main,
            sequence: 1,
            rotation: aim,
        },
    );
    s.tick(25);
    alice.link.take();
    send_input(
        &mut s.server,
        &alice,
        PlayerInput::Dig {
            action: DigAction::ReleaseUseItem,
            pos: BlockPos::new(0, 0, 0),
            face: Direction::Down,
            sequence: 0,
        },
    );
}

fn arrows_held(s: &mut Scene) -> i32 {
    let Answer::Inventory(state) = s.ask(Question::Inventory(s.players[0].id)) else {
        panic!("expected Alice's inventory");
    };
    state
        .slots
        .iter()
        .flatten()
        .filter(|item| item.key == "minecraft:arrow")
        .map(|item| item.count)
        .sum()
}

fn arrows(s: &Scene) -> usize {
    s.server.entities_of(Kind::Arrow).len()
}

fn shot_sounds(player: &Player) -> usize {
    player
        .link
        .take()
        .iter()
        .filter(|event| {
            matches!(event, ClientEvent::Sound { sound, .. } if *sound == "minecraft:entity.arrow.shoot")
        })
        .count()
}

#[test]
fn a_bow_shot_spends_its_arrow_before_it_asks_and_a_cancel_keeps_the_sound() {
    let mut s = waiting_scene(vec![deciding(1, EventKind::EntityShootBow)]);
    let alice = s.players[0].clone();
    shoot(&mut s);
    let asked = last_asked(&s.plugin, EventKind::EntityShootBow);
    let EventPayload::EntityShootBow(shot) = &asked.payload else {
        panic!("a shoot bow");
    };
    assert_eq!(
        shot.bow.as_ref().map(|bow| bow.key.as_str()),
        Some("minecraft:bow")
    );
    assert_eq!(
        shot.consumable
            .as_ref()
            .map(|ammo| (ammo.key.as_str(), ammo.count)),
        Some(("minecraft:arrow", 1))
    );
    assert_eq!(shot.hand, EquipmentSlot::Hand);
    assert_eq!(shot.force, 3.0);
    assert!(shot.consume_item);
    assert_eq!(shot.projectile.kind, "minecraft:arrow");
    assert_eq!(arrows_held(&mut s), 1);
    assert_eq!(arrows(&s), 0);
    rule_on(&mut s.server, 1, asked.id, true, None);
    s.server.deliver();
    assert_eq!(arrows(&s), 0);
    assert_eq!(arrows_held(&mut s), 1);
    assert_eq!(shot_sounds(&alice), 1);
}

#[test]
fn a_settled_shot_launches_its_arrow_unless_a_plugin_replaced_it() {
    let mut s = waiting_scene(vec![
        deciding(1, EventKind::EntityShootBow),
        observe(2, EventKind::EntitySpawn),
    ]);
    shoot(&mut s);
    let asked = last_asked(&s.plugin, EventKind::EntityShootBow);
    rule_on(&mut s.server, 1, asked.id, false, None);
    assert_eq!(arrows(&s), 1);
    assert_eq!(settled_for(&s.plugin, EventKind::ProjectileLaunch).len(), 1);
    let EventPayload::EntityShootBow(shot) = &asked.payload else {
        panic!("a shoot bow");
    };
    let mut other: EntityView = shot.projectile.clone();
    other.id = EntityId(other.id.0 + 1_000);
    other.uuid = Uuid::from_u128(7);
    s.tick(25);
    let alice = s.players[0].clone();
    send_input(
        &mut s.server,
        &alice,
        PlayerInput::UseItem {
            hand: Hand::Main,
            sequence: 2,
            rotation: AIM,
        },
    );
    s.tick(25);
    send_input(
        &mut s.server,
        &alice,
        PlayerInput::Dig {
            action: DigAction::ReleaseUseItem,
            pos: BlockPos::new(0, 0, 0),
            face: Direction::Down,
            sequence: 0,
        },
    );
    let second = last_asked(&s.plugin, EventKind::EntityShootBow);
    assert_ne!(second.id, asked.id);
    rule_on(
        &mut s.server,
        1,
        second.id,
        false,
        Some(EventPatch::EntityShootBow(EntityShootBowPatch {
            projectile: Some(other),
        })),
    );
    assert_eq!(
        settled_for(&s.plugin, EventKind::ProjectileLaunch).len(),
        1,
        "the replaced arrow never launches"
    );
}

#[test]
fn an_arrow_waits_on_its_hit_and_a_cancel_lets_it_fly_on() {
    let mut s = waiting_scene(vec![
        deciding(1, EventKind::ProjectileHit),
        observe(2, EventKind::EntityDamageByEntity),
    ]);
    console(&mut s, "time set 18000");
    console(&mut s, "summon minecraft:zombie 0.5 64 4.5 {NoAI:1b}");
    let cow = s.server.entities_of(Kind::Zombie)[0];
    shoot(&mut s);
    s.tick(10);
    let asked = asked_for(&s.plugin, EventKind::ProjectileHit);
    assert_eq!(asked.len(), 1);
    let EventPayload::ProjectileHit(hit) = &asked[0].payload else {
        panic!("a projectile hit");
    };
    assert_eq!(
        hit.hit_entity.as_ref().map(|entity| entity.id),
        Some(cow.id)
    );
    assert_eq!(hit.hit_block, None);
    let arrow = s.server.entities_of(Kind::Arrow)[0];
    s.tick(5);
    assert_eq!(
        s.server.entities_of(Kind::Arrow)[0].position,
        arrow.position
    );
    rule_on(&mut s.server, 1, asked[0].id, true, None);
    s.tick(3);
    assert!(settled_for(&s.plugin, EventKind::EntityDamageByEntity).is_empty());
    assert!(s.server.entities_of(Kind::Arrow)[0].position.z > arrow.position.z);
    shoot(&mut s);
    s.tick(10);
    let again = last_asked(&s.plugin, EventKind::ProjectileHit);
    assert_ne!(again.id, asked[0].id);
    rule_on(&mut s.server, 1, again.id, false, None);
    let EventPayload::EntityDamageByEntity(damage) =
        &settled_for(&s.plugin, EventKind::EntityDamageByEntity)[0].payload
    else {
        panic!("a damage by entity");
    };
    assert_eq!(damage.damager.kind, "minecraft:arrow");
    assert_eq!(damage.entity.id, cow.id);
}

#[test]
fn a_block_hit_asks_its_deciders_and_drops_their_verdicts() {
    let mut s = scene_with(
        &["Alice"],
        vec![
            deciding(1, EventKind::ProjectileHit),
            observe(2, EventKind::ProjectileHit),
        ],
    );
    for y in 63..=67 {
        for x in -2..=2 {
            s.act(Order::Block {
                pos: BlockPos::new(x, y, 5),
                order: BlockOrder::Set("minecraft:stone".into()),
            });
        }
    }
    shoot(&mut s);
    s.tick(10);
    let asked = asked_for(&s.plugin, EventKind::ProjectileHit);
    assert_eq!(asked.len(), 1);
    let EventPayload::ProjectileHit(hit) = &asked[0].payload else {
        panic!("a projectile hit");
    };
    assert_eq!(hit.hit_entity, None);
    assert_eq!(hit.hit_block.map(|pos| pos.z), Some(5));
    assert_eq!(
        hit.hit_block_face,
        Some(rspigot_core::plugins::events::Face::North)
    );
    let settled = settled_for(&s.plugin, EventKind::ProjectileHit);
    assert_eq!(settled.len(), 1);
    assert_eq!(settled[0].id, asked[0].id);
    let stuck = s.server.entities_of(Kind::Arrow)[0];
    rule_on(&mut s.server, 1, asked[0].id, true, None);
    s.tick(3);
    assert!(s.warnings().is_empty());
    assert_eq!(settled_for(&s.plugin, EventKind::ProjectileHit).len(), 1);
    assert_eq!(
        s.server.entities_of(Kind::Arrow)[0].position,
        stuck.position
    );
}

#[test]
fn a_skeleton_asks_before_it_shoots_and_a_cancel_drops_the_arrow_and_its_sound() {
    let mut s = waiting_scene(vec![deciding(1, EventKind::EntityShootBow)]);
    let alice = s.players[0].clone();
    console(&mut s, "time set 18000");
    stream_chunk_of(&mut s, BlockPos::new(0, 64, 9));
    console(
        &mut s,
        "summon minecraft:skeleton 0.5 64 9.5 {equipment:{mainhand:{id:\"minecraft:bow\",count:1}}}",
    );
    for _ in 0..600 {
        s.tick(1);
        if !asked_for(&s.plugin, EventKind::EntityShootBow).is_empty() {
            break;
        }
    }
    let asked = last_asked(&s.plugin, EventKind::EntityShootBow);
    let EventPayload::EntityShootBow(shot) = &asked.payload else {
        panic!("a shoot bow");
    };
    assert_eq!(shot.entity_type, "minecraft:skeleton");
    assert_eq!(shot.consumable, None);
    assert_eq!(shot.force, 0.8);
    assert_eq!(shot.hand, EquipmentSlot::Hand);
    alice.link.take();
    rule_on(&mut s.server, 1, asked.id, true, None);
    s.server.deliver();
    assert_eq!(arrows(&s), 0);
    assert!(!alice.link.take().iter().any(|event| matches!(
        event,
        ClientEvent::Sound { sound, .. } if *sound == "minecraft:entity.skeleton.shoot"
    )));
}
