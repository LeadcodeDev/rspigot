mod common;

use common::*;
use rspigot_core::config::PluginSettings;
use rspigot_core::events::{Hand, Inbound, PlayerInput};
use rspigot_core::plugins::events::{
    DyeColor, EntityBreedPatch, EntityEnterLoveModePatch, SheepDyeWoolPatch,
};
use rspigot_core::plugins::{Answer, EventKind, EventPatch, EventPayload, HandlerSpec, Question};
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

fn console(s: &mut Scene, line: &str) {
    s.server.handle(Inbound::Console(line.to_owned()));
    s.server.deliver();
}

fn summon(s: &mut Scene, line: &str, kind: Kind) -> Vec<EntityId> {
    stream_chunk_of(s, BlockPos::new(1, 64, 1));
    console(s, line);
    s.server
        .entities_of(kind)
        .into_iter()
        .map(|entity| entity.id)
        .collect()
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

fn in_love(s: &Scene, id: EntityId) -> i32 {
    s.server
        .saved_entity(id)
        .and_then(|nbt| nbt.get_i32("InLove"))
        .unwrap_or(0)
}

fn wool(s: &Scene, id: EntityId) -> i64 {
    s.server
        .saved_entity(id)
        .and_then(|nbt| nbt.get_i64("Color"))
        .unwrap_or(0)
}

fn held_count(s: &mut Scene, key: &str) -> i32 {
    let Answer::Inventory(state) = s.ask(Question::Inventory(s.players[0].id)) else {
        panic!("expected Alice's inventory");
    };
    state
        .slots
        .iter()
        .flatten()
        .filter(|item| item.key == key)
        .map(|item| item.count)
        .sum()
}

#[test]
fn feeding_spends_the_food_before_love_mode_is_asked() {
    let mut s = waiting_scene(vec![deciding(1, EventKind::EntityEnterLoveMode)]);
    console(&mut s, "give Alice minecraft:wheat 3");
    let cow = summon(
        &mut s,
        "summon minecraft:cow 1.5 64 1.5 {NoAI:1b}",
        Kind::Cow,
    )[0];
    interact(&mut s, cow);
    let asked = last_asked(&s.plugin, EventKind::EntityEnterLoveMode);
    let EventPayload::EntityEnterLoveMode(love) = &asked.payload else {
        panic!("a love mode");
    };
    assert_eq!(love.entity.id, cow);
    assert_eq!(love.ticks_in_love, 600);
    assert_eq!(
        love.human_entity
            .as_ref()
            .map(|player| player.name.as_str()),
        Some("Alice")
    );
    assert_eq!(held_count(&mut s, "minecraft:wheat"), 2);
    assert_eq!(in_love(&s, cow), 0);
    rule_on(&mut s.server, 1, asked.id, true, None);
    assert_eq!(in_love(&s, cow), 0);
    assert_eq!(held_count(&mut s, "minecraft:wheat"), 2);
    interact(&mut s, cow);
    let asked = last_asked(&s.plugin, EventKind::EntityEnterLoveMode);
    rule_on(
        &mut s.server,
        1,
        asked.id,
        false,
        Some(EventPatch::EntityEnterLoveMode(EntityEnterLoveModePatch {
            ticks_in_love: Some(50),
        })),
    );
    assert_eq!(in_love(&s, cow), 50);
}

#[test]
fn two_cows_in_love_ask_to_breed_with_their_breeder_and_item() {
    let mut s = waiting_scene(vec![
        deciding(1, EventKind::EntityBreed),
        observe(2, EventKind::CreatureSpawn),
    ]);
    console(&mut s, "give Alice minecraft:wheat 5");
    summon(&mut s, "summon minecraft:cow 1.5 64 1.5", Kind::Cow);
    console(&mut s, "summon minecraft:cow 2.5 64 1.5");
    let all: Vec<EntityId> = s
        .server
        .entities_of(Kind::Cow)
        .into_iter()
        .map(|cow| cow.id)
        .collect();
    let cows = [all[0], all[1]];
    for cow in cows {
        interact(&mut s, cow);
    }
    for _ in 0..400 {
        s.tick(1);
        if !asked_for(&s.plugin, EventKind::EntityBreed).is_empty() {
            break;
        }
    }
    let asked = last_asked(&s.plugin, EventKind::EntityBreed);
    let EventPayload::EntityBreed(breed) = &asked.payload else {
        panic!("a breed");
    };
    assert!(cows.contains(&breed.mother.id) && cows.contains(&breed.father.id));
    assert_eq!(breed.entity_type, "minecraft:cow");
    assert!((1..=7).contains(&breed.experience));
    assert_eq!(
        breed.breeder.as_ref().map(|player| player.kind.as_str()),
        Some("minecraft:player")
    );
    assert_eq!(
        breed.bred_with.as_ref().map(|item| item.key.as_str()),
        Some("minecraft:wheat")
    );
    assert_eq!(s.server.entities_of(Kind::Cow).len(), 2);
    rule_on(
        &mut s.server,
        1,
        asked.id,
        false,
        Some(EventPatch::EntityBreed(EntityBreedPatch {
            experience: Some(0),
        })),
    );
    assert_eq!(s.server.entities_of(Kind::Cow).len(), 3);
    assert!(s.server.experience_orbs().is_empty());
    assert!(in_love(&s, cows[0]) <= 0 && in_love(&s, cows[1]) <= 0);
}

#[test]
fn a_dyed_sheep_takes_the_settled_color_and_a_cancel_keeps_the_dye() {
    let mut s = waiting_scene(vec![deciding(1, EventKind::SheepDyeWool)]);
    console(&mut s, "give Alice minecraft:red_dye 2");
    let sheep = summon(
        &mut s,
        "summon minecraft:sheep 1.5 64 1.5 {NoAI:1b,Color:0b}",
        Kind::Sheep,
    )[0];
    interact(&mut s, sheep);
    let asked = last_asked(&s.plugin, EventKind::SheepDyeWool);
    let EventPayload::SheepDyeWool(dye) = &asked.payload else {
        panic!("a sheep dye");
    };
    assert_eq!(dye.color, DyeColor::Red);
    assert_eq!(dye.entity.id, sheep);
    assert_eq!(wool(&s, sheep), 0);
    rule_on(&mut s.server, 1, asked.id, true, None);
    assert_eq!(wool(&s, sheep), 0);
    assert_eq!(held_count(&mut s, "minecraft:red_dye"), 2);
    interact(&mut s, sheep);
    let asked = last_asked(&s.plugin, EventKind::SheepDyeWool);
    rule_on(
        &mut s.server,
        1,
        asked.id,
        false,
        Some(EventPatch::SheepDyeWool(SheepDyeWoolPatch {
            color: Some(DyeColor::Blue),
        })),
    );
    assert_eq!(wool(&s, sheep), 11);
    assert_eq!(held_count(&mut s, "minecraft:red_dye"), 1);
}
