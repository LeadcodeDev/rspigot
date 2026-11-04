mod common;

use std::sync::Arc;
use std::time::Duration;

use common::*;
use rspigot_core::config::{GameConfig, PluginSettings};
use rspigot_core::plugins::events::EntityRemoveCause;
use rspigot_core::plugins::{
    Answer, EntityView, EventKind, EventPayload, HandlerSpec, InventoryState, ItemSpec, Question,
};
use rspigot_core::server::Server;
use rspigot_core::status::StatusBoard;
use rspigot_domain::Vec3;

fn ground_scene(handlers: Vec<HandlerSpec>, despawn: u32) -> Scene {
    let config = GameConfig {
        view_distance: 3,
        chunk_workers: 2,
        spawn_protection: 0,
        item_despawn_ticks: despawn,
        plugins: PluginSettings {
            decision_ticks: 1_000,
            ..PluginSettings::default()
        },
        ..GameConfig::default()
    };
    let mut server: Server<Link> = Server::new(
        config,
        rspigot_core::config::LevelData::default(),
        Arc::new(NoStore),
        Arc::new(Floor),
        StatusBoard::new(20),
    );
    server.wait_for_spawn(Duration::from_secs(10));
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

fn drop_stick(s: &mut Scene, position: Vec3) -> EntityView {
    let Answer::Spawned(view) = s.ask(Question::DropItem {
        position,
        item: ItemSpec::of("minecraft:stick", 3),
    }) else {
        panic!("expected the dropped stick");
    };
    view
}

fn on_ground(s: &Scene) -> usize {
    s.server.items_on_ground()
}

fn inventory(s: &mut Scene) -> InventoryState {
    let Answer::Inventory(state) = s.ask(Question::Inventory(s.players[0].id)) else {
        panic!("expected Alice's inventory");
    };
    state
}

fn sticks(s: &mut Scene) -> i32 {
    inventory(s)
        .slots
        .iter()
        .flatten()
        .filter(|item| item.key == "minecraft:stick")
        .map(|item| item.count)
        .sum()
}

#[test]
fn an_old_item_asks_to_despawn_once_and_a_cancel_restarts_its_age() {
    let mut s = ground_scene(
        vec![
            deciding(1, EventKind::ItemDespawn),
            observe(2, EventKind::EntityRemove),
        ],
        20,
    );
    let stick = drop_stick(&mut s, Vec3::new(9.5, 64.0, 9.5));
    s.tick(30);
    let asked = asked_for(&s.plugin, EventKind::ItemDespawn);
    assert_eq!(asked.len(), 1);
    let EventPayload::ItemDespawn(despawn) = &asked[0].payload else {
        panic!("an item despawn");
    };
    assert_eq!(despawn.entity.id, stick.id);
    assert_eq!(despawn.entity_type, "minecraft:item");
    rule_on(&mut s.server, 1, asked[0].id, true, None);
    assert_eq!(on_ground(&s), 1);
    s.tick(15);
    assert_eq!(asked_for(&s.plugin, EventKind::ItemDespawn).len(), 1);
    s.tick(10);
    let again = last_asked(&s.plugin, EventKind::ItemDespawn);
    rule_on(&mut s.server, 1, again.id, false, None);
    assert_eq!(on_ground(&s), 0);
    let removed: Vec<EntityRemoveCause> = fired(&s.plugin)
        .into_iter()
        .filter_map(|event| match event.payload {
            EventPayload::EntityRemove(removed) => Some(removed.cause),
            _ => None,
        })
        .collect();
    assert_eq!(removed, [EntityRemoveCause::Despawn]);
}

#[test]
fn a_pickup_is_asked_once_while_it_waits_and_a_cancel_leaves_the_item() {
    let mut s = ground_scene(
        vec![
            deciding(1, EventKind::EntityPickupItem),
            observe(2, EventKind::EntityRemove),
        ],
        6_000,
    );
    let stick = drop_stick(&mut s, Vec3::new(0.5, 64.0, 0.5));
    s.tick(20);
    let asked = asked_for(&s.plugin, EventKind::EntityPickupItem);
    assert_eq!(asked.len(), 1);
    let EventPayload::EntityPickupItem(pickup) = &asked[0].payload else {
        panic!("an entity pickup");
    };
    assert_eq!(pickup.item.id, stick.id);
    assert_eq!(pickup.entity.kind, "minecraft:player");
    assert_eq!(pickup.remaining, 0);
    assert_eq!(sticks(&mut s), 0);
    rule_on(&mut s.server, 1, asked[0].id, true, None);
    assert_eq!(on_ground(&s), 1);
    assert_eq!(sticks(&mut s), 0);
    s.tick(1);
    let again = last_asked(&s.plugin, EventKind::EntityPickupItem);
    assert_ne!(again.id, asked[0].id);
    rule_on(&mut s.server, 1, again.id, false, None);
    assert_eq!(on_ground(&s), 0);
    assert_eq!(sticks(&mut s), 3);
    let kinds: Vec<EventKind> = fired(&s.plugin)
        .into_iter()
        .map(|event| event.payload.kind())
        .collect();
    assert_eq!(kinds.last(), Some(&EventKind::EntityRemove));
}
