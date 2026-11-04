mod common;

use common::*;
use rspigot_core::events::{Hand, Inbound, PlayerInput};
use rspigot_core::plugins::events::{PlayerPickupArrow, PlayerShearEntity};
use rspigot_core::plugins::{Answer, EventKind, EventPayload, ItemSpec, Phase, Question};
use rspigot_domain::{EntityId, Vec3};
use rspigot_entities::EntityKind as Kind;

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
        .expect("summoned")
}

fn arrows_held(s: &mut Scene, player: &Player) -> i32 {
    match s.ask(Question::Inventory(player.id)) {
        Answer::Inventory(inventory) => inventory
            .slots
            .iter()
            .flatten()
            .filter(|item| item.key == "minecraft:arrow")
            .map(|item| item.count)
            .sum(),
        other => panic!("expected the inventory, got {other:?}"),
    }
}

fn walk_to(s: &mut Scene, player: &Player, position: Vec3) {
    send_input(
        &mut s.server,
        player,
        PlayerInput::Move {
            position: Some(position),
            rotation: None,
            on_ground: true,
        },
    );
}

fn landed_arrow(s: &mut Scene) -> EntityId {
    let arrow = summoned(s, "summon arrow 2.5 67 0.5 {pickup:1b}", Kind::Arrow);
    s.tick(40);
    assert!(s.server.arrows().iter().all(|(_, _, in_ground)| *in_ground));
    arrow
}

#[test]
fn an_arrow_pickup_is_asked_once_at_a_time_and_a_cancel_leaves_it_on_the_ground() {
    let mut s = scene_with(&["Alice"], vec![deciding(1, EventKind::PlayerPickupArrow)]);
    let alice = s.players[0].clone();
    let arrow = landed_arrow(&mut s);
    walk_to(&mut s, &alice, Vec3::new(2.5, 64.0, 0.5));
    s.server.tick();
    s.server.tick();
    s.server.deliver();
    let asked = asked_for(&s.plugin, EventKind::PlayerPickupArrow);
    assert_eq!(asked.len(), 1);
    let EventPayload::PlayerPickupArrow(PlayerPickupArrow {
        arrow: seen,
        item,
        remaining,
        ..
    }) = &asked[0].payload
    else {
        panic!("expected an arrow pickup");
    };
    assert_eq!(seen.id, arrow);
    assert_eq!(item.kind, "minecraft:item");
    assert_eq!(*remaining, 0);
    rule_on(&mut s.server, 1, asked[0].id, true, None);
    assert_eq!(arrows_held(&mut s, &alice), 0);
    assert_eq!(s.server.arrows().len(), 1);
    s.server.tick();
    s.server.deliver();
    let asked = asked_for(&s.plugin, EventKind::PlayerPickupArrow);
    assert_eq!(asked.len(), 2);
    rule_on(&mut s.server, 1, asked[1].id, false, None);
    assert_eq!(arrows_held(&mut s, &alice), 1);
    assert!(s.server.arrows().is_empty());
}

#[test]
fn an_arrow_pickup_without_a_decider_happens_at_once() {
    let mut s = scene_with(&["Alice"], vec![observe(1, EventKind::PlayerPickupArrow)]);
    let alice = s.players[0].clone();
    landed_arrow(&mut s);
    walk_to(&mut s, &alice, Vec3::new(2.5, 64.0, 0.5));
    s.tick(2);
    assert_eq!(arrows_held(&mut s, &alice), 1);
    assert!(asked_for(&s.plugin, EventKind::PlayerPickupArrow).is_empty());
    assert_eq!(
        settled_for(&s.plugin, EventKind::PlayerPickupArrow).len(),
        1
    );
}

fn shear(s: &mut Scene, player: &Player, sheep: EntityId) {
    send_input(
        &mut s.server,
        player,
        PlayerInput::Interact {
            target: sheep,
            hand: Hand::Main,
            sneaking: false,
            location: Vec3::new(0.0, 0.5, 0.0),
        },
    );
}

fn wool_on_the_ground(s: &Scene) -> usize {
    s.server.items_on_ground()
}

#[test]
fn shearing_fires_the_click_then_the_shear_then_the_wear_and_a_cancel_keeps_the_wool() {
    let mut s = scene_with(
        &["Alice"],
        vec![
            observe(1, EventKind::PlayerInteractAtEntity),
            deciding(2, EventKind::PlayerShearEntity),
            observe(3, EventKind::PlayerItemDamage),
            observe(4, EventKind::PlayerShearEntity),
        ],
    );
    let alice = s.players[0].clone();
    console(&mut s, "give Alice shears 1");
    let sheep = summoned(&mut s, "summon sheep 1.5 64 0.5 {NoAI:1b}", Kind::Sheep);
    shear(&mut s, &alice, sheep);
    let asked = last_asked(&s.plugin, EventKind::PlayerShearEntity);
    let EventPayload::PlayerShearEntity(PlayerShearEntity { entity, item, .. }) = &asked.payload
    else {
        panic!("expected a shear");
    };
    assert_eq!(entity.id, sheep);
    assert_eq!(item.key, "minecraft:shears");
    judge(&mut s.server, 1, asked.id, 2, true, None);
    assert_eq!(wool_on_the_ground(&s), 0);
    assert!(settled_for(&s.plugin, EventKind::PlayerItemDamage).is_empty());
    shear(&mut s, &alice, sheep);
    let asked = last_asked(&s.plugin, EventKind::PlayerShearEntity);
    judge(&mut s.server, 1, asked.id, 2, false, None);
    assert!(wool_on_the_ground(&s) > 0);
    let order: Vec<(Phase, EventKind)> = fired_kinds(&s.plugin)
        .into_iter()
        .filter(|(phase, _)| *phase == Phase::Settled)
        .collect();
    assert_eq!(
        order,
        [
            (Phase::Settled, EventKind::PlayerInteractAtEntity),
            (Phase::Settled, EventKind::PlayerShearEntity),
            (Phase::Settled, EventKind::PlayerInteractAtEntity),
            (Phase::Settled, EventKind::PlayerShearEntity),
            (Phase::Settled, EventKind::PlayerItemDamage),
        ]
    );
    let held = match s.ask(Question::Inventory(alice.id)) {
        Answer::Inventory(inventory) => inventory.slots[0].clone(),
        other => panic!("expected the inventory, got {other:?}"),
    };
    assert_ne!(held, Some(ItemSpec::of("minecraft:shears", 1)));
}

#[test]
fn a_busy_lane_without_an_arrow_listener_still_picks_up_arrows() {
    let mut s = scene_with(&["Alice"], vec![decide(1)]);
    let alice = s.players[0].clone();
    landed_arrow(&mut s);
    walk_to(&mut s, &alice, Vec3::new(2.5, 64.0, 0.5));
    let walked = last_asked(&s.plugin, EventKind::PlayerMove);
    rule_on(&mut s.server, 1, walked.id, false, None);
    walk_to(&mut s, &alice, Vec3::new(2.6, 64.0, 0.5));
    let pending = last_asked(&s.plugin, EventKind::PlayerMove);
    assert_ne!(pending.id, walked.id);
    s.server.tick();
    s.server.deliver();
    assert_eq!(arrows_held(&mut s, &alice), 1);
}
