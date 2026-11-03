mod common;

use common::*;
use rspigot_core::events::{ClientEvent, Hand, Inbound, PlayerInput};
use rspigot_core::plugins::{
    Answer, EffectSpec, EntityDetails, EntityOrder, EntityView, EventKind, EventPayload, ItemSpec,
    Order, Question, ToPlugin, WorldOrder,
};
use rspigot_domain::world::Difficulty;
use rspigot_domain::{EntityId, Vec3};
use rspigot_items::ItemKind;

fn spawn_answer(s: &mut Scene, kind: &str, position: Vec3) -> Answer {
    s.ask(Question::Spawn {
        kind: kind.into(),
        position,
    })
}

fn spawn(s: &mut Scene, kind: &str, position: Vec3) -> EntityView {
    let Answer::Spawned(view) = spawn_answer(s, kind, position) else {
        panic!("expected the spawned {kind}");
    };
    view
}

fn listed(s: &mut Scene, near: Option<(Vec3, f64)>, kind: Option<&str>) -> Vec<EntityId> {
    let Answer::Entities(views) = s.ask(Question::Entities {
        near,
        kind: kind.map(Into::into),
    }) else {
        panic!("expected a list of entities");
    };
    views.into_iter().map(|view| view.id).collect()
}

fn details(s: &mut Scene, id: EntityId) -> EntityDetails {
    let Answer::Entity(details) = s.ask(Question::Entity(id)) else {
        panic!("expected the entity {id:?}");
    };
    details
}

fn effects(s: &Scene, id: EntityId) -> Vec<String> {
    let saved = s.server.saved_entity(id).expect("the entity is saved");
    saved
        .get_list("active_effects")
        .unwrap_or_default()
        .iter()
        .filter_map(|effect| effect.as_compound()?.get_str("id").map(str::to_owned))
        .collect()
}

fn slowness() -> EffectSpec {
    EffectSpec {
        key: "minecraft:slowness".into(),
        duration: 200,
        amplifier: 1,
        ambient: false,
        particles: true,
        icon: true,
    }
}

fn spawned_to(player: &Player, id: EntityId) -> bool {
    player
        .link
        .take()
        .iter()
        .any(|event| matches!(event, ClientEvent::SpawnEntity { entity, .. } if *entity == id))
}

#[test]
fn a_spawned_zombie_is_listed_near_its_position() {
    let mut s = scene(&["Alice"]);
    let alice = s.players[0].clone();
    let at = Vec3::new(4.5, 64.0, 4.5);
    stream_chunk_of(&mut s, at.block());
    let Answer::Spawned(zombie) = spawn_answer(&mut s, "minecraft:zombie", at) else {
        panic!("expected the spawned zombie");
    };
    assert_eq!(zombie.kind, "minecraft:zombie");
    assert_eq!(zombie.position, at);
    assert!(listed(&mut s, Some((at, 2.0)), None).contains(&zombie.id));
    assert!(listed(&mut s, Some((Vec3::new(100.0, 64.0, 100.0), 2.0)), None).is_empty());
    assert!(listed(&mut s, None, Some("minecraft:cow")).is_empty());
    assert_eq!(
        listed(&mut s, Some((at, 2.0)), Some("minecraft:zombie")),
        [zombie.id]
    );
    s.tick(1);
    assert!(spawned_to(&alice, zombie.id));
    assert!(s.warnings().is_empty());
}

#[test]
fn spawning_is_refused_with_the_vanilla_reason() {
    let mut s = scene(&["Alice"]);
    s.act(Order::World(WorldOrder::SetDifficulty(
        Difficulty::Peaceful,
    )));
    let at = Vec3::new(4.5, 64.0, 4.5);
    assert_eq!(
        spawn_answer(&mut s, "minecraft:zombie", at),
        Answer::Refused("Monsters cannot be summoned in Peaceful difficulty".into())
    );
    assert_eq!(
        spawn_answer(&mut s, "minecraft:player", at),
        Answer::Refused("Unable to summon entity".into())
    );
    assert!(matches!(
        spawn_answer(&mut s, "minecraft:nope", at),
        Answer::Refused(_)
    ));
    let before = s.server.entity_count();
    assert_eq!(
        spawn_answer(&mut s, "minecraft:cow", Vec3::new(100_000.0, 64.0, 0.0)),
        Answer::Refused("Unable to summon entity".into())
    );
    assert!(matches!(
        spawn_answer(&mut s, "minecraft:cow", Vec3::new(f64::NAN, 64.0, 0.0)),
        Answer::Refused(_)
    ));
    assert_eq!(s.server.entity_count(), before);
    let high = spawn(&mut s, "minecraft:cow", Vec3::new(4.5, 400.0, 4.5));
    assert_eq!(high.position.y, 400.0);
    assert!(s.warnings().is_empty());
}

#[test]
fn entities_do_not_list_players() {
    let mut s = scene(&["Alice", "Bob"]);
    let cow = spawn(&mut s, "minecraft:cow", Vec3::new(2.5, 64.0, 2.5));
    let Answer::Players(players) = s.ask(Question::Players) else {
        panic!("expected the players");
    };
    let all = listed(&mut s, None, None);
    assert!(all.contains(&cow.id));
    for player in &players {
        let entity = EntityId(player.entity_id);
        assert!(!all.contains(&entity));
        assert!(!listed(&mut s, Some((player.position, 8.0)), None).contains(&entity));
        assert_eq!(s.ask(Question::Entity(entity)), Answer::NotFound);
        s.act(Order::Entity {
            entity,
            order: EntityOrder::Damage(5.0),
        });
        let Answer::Player(state) = s.ask(Question::Player(player.id)) else {
            panic!("expected {}", player.name);
        };
        assert_eq!(state.health, 20.0);
    }
    assert!(s.warnings().is_empty());
}

#[test]
fn entity_orders_use_the_command_paths() {
    let mut s = scene(&["Alice"]);
    let alice = s.players[0].clone();
    let at = Vec3::new(4.5, 64.0, 4.5);
    stream_chunk_of(&mut s, at.block());
    let creeper = spawn(&mut s, "minecraft:creeper", at);
    let to = |order| Order::Entity {
        entity: creeper.id,
        order,
    };
    s.tick(1);
    assert!(spawned_to(&alice, creeper.id));

    let there = Vec3::new(6.5, 64.0, 6.5);
    s.act(to(EntityOrder::Teleport(there)));
    assert_eq!(details(&mut s, creeper.id).entity.position, there);
    s.act(to(EntityOrder::SetVelocity(Vec3::new(0.0, 0.5, 0.0))));
    assert_eq!(details(&mut s, creeper.id).velocity.y, 0.5);
    s.act(to(EntityOrder::Damage(5.0)));
    assert_eq!(details(&mut s, creeper.id).health, Some(15.0));

    s.act(to(EntityOrder::AddEffect(slowness())));
    assert_eq!(effects(&s, creeper.id), ["minecraft:slowness"]);
    s.act(to(EntityOrder::RemoveEffect("minecraft:slowness".into())));
    assert!(effects(&s, creeper.id).is_empty());
    s.act(to(EntityOrder::AddEffect(slowness())));
    s.act(to(EntityOrder::ClearEffects));
    assert!(effects(&s, creeper.id).is_empty());
    assert!(s.warnings().is_empty());

    alice.link.take();
    s.act(to(EntityOrder::Remove));
    assert_eq!(s.ask(Question::Entity(creeper.id)), Answer::NotFound);
    assert!(
        alice
            .link
            .take()
            .contains(&ClientEvent::RemoveEntities(vec![creeper.id]))
    );
    s.act(to(EntityOrder::Teleport(there)));
    assert!(s.warnings().is_empty());

    let cow = spawn(&mut s, "minecraft:cow", Vec3::new(2.5, 64.0, 2.5));
    s.act(Order::Entity {
        entity: cow.id,
        order: EntityOrder::Kill,
    });
    assert!(matches!(
        s.ask(Question::Entity(cow.id)),
        Answer::NotFound
            | Answer::Entity(EntityDetails {
                health: Some(0.0),
                ..
            })
    ));
    s.tick(30);
    assert_eq!(s.ask(Question::Entity(cow.id)), Answer::NotFound);
    assert!(s.warnings().is_empty());
}

#[test]
fn drop_item_returns_the_item_entity() {
    let mut s = scene(&["Alice"]);
    let Answer::Spawned(view) = s.ask(Question::DropItem {
        position: Vec3::new(2.5, 65.0, 2.5),
        item: ItemSpec::of("minecraft:diamond", 3),
    }) else {
        panic!("expected the dropped item");
    };
    assert_eq!(view.kind, "minecraft:item");
    assert_eq!(view.position, Vec3::new(2.5, 65.0, 2.5));
    assert!(
        s.server
            .ground_items()
            .iter()
            .any(|(id, stack)| *id == view.id
                && stack.item == ItemKind::Diamond.id()
                && stack.count.get() == 3)
    );
    let velocity = details(&mut s, view.id).velocity;
    assert_eq!(velocity.y, 0.2);
    assert!((-0.1..0.1).contains(&velocity.x));
    assert!((-0.1..0.1).contains(&velocity.z));

    for item in [
        ItemSpec::of("minecraft:air", 1),
        ItemSpec::of("minecraft:diamond", 0),
    ] {
        assert!(matches!(
            s.ask(Question::DropItem {
                position: Vec3::new(2.5, 65.0, 2.5),
                item,
            }),
            Answer::Refused(_)
        ));
    }
    assert!(matches!(
        s.ask(Question::DropItem {
            position: Vec3::new(100_000.0, 65.0, 0.0),
            item: ItemSpec::of("minecraft:diamond", 1),
        }),
        Answer::Refused(_)
    ));
    assert_eq!(s.server.ground_items().len(), 1);
}

#[test]
fn a_dropped_item_waits_ten_ticks_before_it_can_be_picked_up() {
    let mut s = scene(&["Alice"]);
    let Answer::Players(players) = s.ask(Question::Players) else {
        panic!("expected the players");
    };
    let feet = players[0].position;
    let Answer::Spawned(view) = s.ask(Question::DropItem {
        position: feet,
        item: ItemSpec::of("minecraft:diamond", 1),
    }) else {
        panic!("expected the dropped item");
    };
    let on_the_ground = |s: &Scene| s.server.ground_items().iter().any(|(id, _)| *id == view.id);
    s.tick(9);
    assert!(on_the_ground(&s));
    s.tick(3);
    assert!(!on_the_ground(&s));
}

#[test]
fn invalid_entity_orders_warn() {
    let mut s = scene(&["Alice"]);
    let creeper = spawn(&mut s, "minecraft:creeper", Vec3::new(4.5, 64.0, 4.5));
    let before = details(&mut s, creeper.id);
    let unknown = EffectSpec {
        key: "minecraft:nope".into(),
        ..slowness()
    };
    let bad_orders = [
        EntityOrder::Damage(-1.0),
        EntityOrder::Damage(f64::NAN),
        EntityOrder::Teleport(Vec3::new(4.0e7, 64.0, 0.0)),
        EntityOrder::Teleport(Vec3::new(f64::NAN, 64.0, 0.0)),
        EntityOrder::SetVelocity(Vec3::new(f64::NAN, 0.0, 0.0)),
        EntityOrder::AddEffect(unknown),
    ];
    for order in bad_orders {
        let order = Order::Entity {
            entity: creeper.id,
            order,
        };
        let name = order.name();
        assert!(name.starts_with("entity."), "{name}");
        let count = s.warnings().len();
        s.act(order);
        let warnings = s.warnings();
        assert_eq!(warnings.len(), count + 1, "{name}");
        let last = warnings.last().unwrap();
        assert!(
            last.starts_with(&format!("{name} was not applied: ")),
            "{last}"
        );
    }
    assert_eq!(details(&mut s, creeper.id), before);
    assert!(effects(&s, creeper.id).is_empty());
}

#[test]
fn invalid_entity_questions_are_refused() {
    let mut s = scene(&["Alice"]);
    let at = Vec3::new(4.5, 64.0, 4.5);
    for (near, kind) in [
        (Some((at, -1.0)), None),
        (Some((at, f64::NAN)), None),
        (Some((at, f64::INFINITY)), None),
        (Some((Vec3::new(f64::NAN, 64.0, 4.5), 2.0)), None),
        (None, Some("minecraft:nope")),
    ] {
        assert!(matches!(
            s.ask(Question::Entities {
                near,
                kind: kind.map(Into::into),
            }),
            Answer::Refused(_)
        ));
    }
    assert_eq!(s.ask(Question::Entity(EntityId(999_999))), Answer::NotFound);
    assert!(s.warnings().is_empty());
}

#[test]
fn appearance_orders_name_and_freeze_a_mob() {
    let mut s = scene(&["Alice"]);
    let at = Vec3::new(4.5, 64.0, 4.5);
    stream_chunk_of(&mut s, at.block());
    let guide = spawn(&mut s, "minecraft:villager", at);
    let to = |order| Order::Entity {
        entity: guide.id,
        order,
    };
    assert_eq!(details(&mut s, guide.id).custom_name, None);

    s.act(to(EntityOrder::SetCustomName("§aGuide".into())));
    s.act(to(EntityOrder::SetNameVisible(true)));
    s.act(to(EntityOrder::SetNoAi(true)));
    s.act(to(EntityOrder::SetInvulnerable(true)));
    s.act(to(EntityOrder::SetSilent(true)));
    assert_eq!(
        details(&mut s, guide.id).custom_name.as_deref(),
        Some("§aGuide")
    );
    let saved = s
        .server
        .saved_entity(guide.id)
        .expect("the entity is saved");
    assert_eq!(saved.get_bool("CustomNameVisible"), Some(true));
    assert_eq!(saved.get_bool("NoAI"), Some(true));
    assert_eq!(saved.get_bool("Invulnerable"), Some(true));
    assert_eq!(saved.get_bool("Silent"), Some(true));

    s.act(to(EntityOrder::Damage(5.0)));
    assert_eq!(details(&mut s, guide.id).health, Some(20.0));

    s.act(to(EntityOrder::ClearCustomName));
    assert_eq!(details(&mut s, guide.id).custom_name, None);
    s.act(to(EntityOrder::SetNoAi(false)));
    s.act(to(EntityOrder::SetInvulnerable(false)));
    s.act(to(EntityOrder::Damage(5.0)));
    assert_eq!(details(&mut s, guide.id).health, Some(15.0));
    assert!(s.warnings().is_empty());
}

#[test]
fn only_a_mob_can_lose_its_ai() {
    let mut s = scene(&["Alice"]);
    let at = Vec3::new(4.5, 64.0, 4.5);
    stream_chunk_of(&mut s, at.block());
    let Answer::Spawned(item) = s.ask(Question::DropItem {
        position: at,
        item: ItemSpec::of("minecraft:diamond", 1),
    }) else {
        panic!("expected the dropped item");
    };
    let count = s.warnings().len();
    s.act(Order::Entity {
        entity: item.id,
        order: EntityOrder::SetNoAi(true),
    });
    assert_eq!(s.warnings().len(), count + 1);
}

fn interactions(s: &Scene) -> Vec<(Vec<u32>, EntityView, Hand, bool)> {
    s.plugin
        .received()
        .into_iter()
        .filter_map(|message| match message {
            ToPlugin::Event {
                handlers,
                payload:
                    EventPayload::EntityInteract {
                        entity,
                        hand,
                        sneaking,
                        ..
                    },
                ..
            } => Some((handlers, entity, hand, sneaking)),
            _ => None,
        })
        .collect()
}

fn interact(s: &mut Scene, target: EntityId, hand: Hand, sneaking: bool) {
    s.server.handle(Inbound::Input {
        player: s.players[0].id,
        input: PlayerInput::Interact {
            target,
            hand,
            sneaking,
        },
    });
    s.server.deliver();
}

#[test]
fn a_plugin_hears_when_a_player_interacts_with_an_entity() {
    let mut s = scene_with(&["Alice"], vec![observe(1, EventKind::EntityInteract)]);
    let Answer::Player(alice) = s.ask(Question::Player(s.players[0].id)) else {
        panic!("expected Alice");
    };
    let mut at = alice.player.position;
    at.x += 1.5;
    let guide = spawn(&mut s, "minecraft:villager", at);

    interact(&mut s, guide.id, Hand::Main, false);
    interact(&mut s, guide.id, Hand::Off, true);
    let heard = interactions(&s);
    assert_eq!(heard.len(), 2);
    assert_eq!(heard[0].0, [1]);
    assert_eq!(heard[0].1.id, guide.id);
    assert_eq!(heard[0].1.kind, "minecraft:villager");
    assert_eq!((heard[0].2, heard[0].3), (Hand::Main, false));
    assert_eq!((heard[1].2, heard[1].3), (Hand::Off, true));
}

#[test]
fn an_unreachable_entity_is_not_heard() {
    let mut s = scene_with(&["Alice"], vec![observe(1, EventKind::EntityInteract)]);
    let Answer::Player(alice) = s.ask(Question::Player(s.players[0].id)) else {
        panic!("expected Alice");
    };
    let mut at = alice.player.position;
    at.x += 30.0;
    stream_chunk_of(&mut s, at.block());
    let guide = spawn(&mut s, "minecraft:villager", at);
    interact(&mut s, guide.id, Hand::Main, false);
    assert!(interactions(&s).is_empty());
}
