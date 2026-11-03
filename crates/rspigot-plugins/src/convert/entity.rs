use rspigot_core::events::Hand;
use rspigot_core::plugins::{
    EntityDetails, EntityOrder, EntityView, Order, PlayerSnapshot, Question,
};
use rspigot_domain::EntityId;
use rspigot_plugin_protocol::v1::{self, entity_order::Order as Wire, event};

use super::Malformed;
use super::player::wire_player;
use super::shared::{self, required, wire_vec};

pub(super) fn order(order: v1::EntityOrder) -> Result<Order, Malformed> {
    let entity = EntityId(order.entity);
    let order = match required(order.order)? {
        Wire::Teleport(position) => EntityOrder::Teleport(shared::vec3(position)),
        Wire::Remove(_) => EntityOrder::Remove,
        Wire::Damage(amount) => EntityOrder::Damage(amount),
        Wire::Kill(_) => EntityOrder::Kill,
        Wire::AddEffect(effect) => EntityOrder::AddEffect(shared::effect(effect)),
        Wire::RemoveEffect(key) => EntityOrder::RemoveEffect(key),
        Wire::ClearEffects(_) => EntityOrder::ClearEffects,
        Wire::SetVelocity(velocity) => EntityOrder::SetVelocity(shared::vec3(velocity)),
        Wire::SetCustomName(name) => EntityOrder::SetCustomName(name),
        Wire::ClearCustomName(_) => EntityOrder::ClearCustomName,
        Wire::SetNameVisible(visible) => EntityOrder::SetNameVisible(visible),
        Wire::SetNoAi(no_ai) => EntityOrder::SetNoAi(no_ai),
        Wire::SetInvulnerable(invulnerable) => EntityOrder::SetInvulnerable(invulnerable),
        Wire::SetSilent(silent) => EntityOrder::SetSilent(silent),
    };
    Ok(Order::Entity { entity, order })
}

pub(super) fn query(query: v1::EntityQuery) -> Result<Question, Malformed> {
    let near = match query.near {
        Some(near) => Some((shared::vec3(required(near.position)?), near.radius)),
        None => None,
    };
    Ok(Question::Entities {
        near,
        kind: query.kind,
    })
}

pub(super) fn spawn(spawn: v1::Spawn) -> Result<Question, Malformed> {
    Ok(Question::Spawn {
        kind: spawn.kind,
        position: shared::vec3(required(spawn.position)?),
    })
}

pub(super) fn drop_item(drop: v1::DropItem) -> Result<Question, Malformed> {
    Ok(Question::DropItem {
        position: shared::vec3(required(drop.position)?),
        item: shared::item(required(drop.item)?),
    })
}

pub(super) fn wire_interact(
    player: PlayerSnapshot,
    entity: EntityView,
    hand: Hand,
    sneaking: bool,
) -> event::Payload {
    let hand = match hand {
        Hand::Main => v1::Hand::Main,
        Hand::Off => v1::Hand::Off,
    };
    event::Payload::EntityInteract(v1::EntityInteract {
        player: Some(wire_player(player)),
        entity: Some(wire_view(entity)),
        hand: hand as i32,
        sneaking,
    })
}

pub(super) fn wire_view(view: EntityView) -> v1::EntitySnapshot {
    v1::EntitySnapshot {
        entity_id: view.id.0,
        uuid: view.uuid.hyphenated().to_string(),
        kind: view.kind,
        position: Some(wire_vec(view.position)),
    }
}

pub(super) fn wire_details(details: EntityDetails) -> v1::EntityState {
    v1::EntityState {
        entity: Some(wire_view(details.entity)),
        health: details.health,
        velocity: Some(wire_vec(details.velocity)),
        on_ground: details.on_ground,
        custom_name: details.custom_name,
        fire_ticks: details.fire_ticks,
    }
}

#[cfg(test)]
mod tests {
    use rspigot_core::events::Hand;
    use rspigot_core::plugins::{
        Answer, EntityDetails, EntityOrder, EntityView, EventPayload, FromPlugin, ItemSpec, Order,
        Phase, PlayerSnapshot, Question, ToPlugin,
    };
    use rspigot_domain::player::{GameMode, PlayerId};
    use rspigot_domain::{EntityId, Rotation, Vec3};
    use rspigot_plugin_protocol::v1::{self, answer, entity_order, event};
    use uuid::Uuid;

    use crate::convert::{Malformed, from_plugin, question, to_plugin};

    const ZOMBIE: EntityId = EntityId(42);

    fn body(body: from_plugin::Body) -> v1::FromPlugin {
        v1::FromPlugin { body: Some(body) }
    }

    fn entity(order: Option<entity_order::Order>) -> Result<FromPlugin, Malformed> {
        from_plugin(body(from_plugin::Body::Entity(v1::EntityOrder {
            entity: ZOMBIE.0,
            order,
        })))
    }

    fn ask(asked: question::Question) -> Result<FromPlugin, Malformed> {
        from_plugin(body(from_plugin::Body::Question(v1::Question {
            request: 9,
            question: Some(asked),
        })))
    }

    fn asked(question: Question) -> FromPlugin {
        FromPlugin::Question {
            request: 9,
            question,
        }
    }

    fn answered(answer: Answer) -> Option<answer::Body> {
        let message = to_plugin(ToPlugin::Answer { request: 9, answer });
        match message.body {
            Some(to_plugin::Body::Answer(v1::Answer { request: 9, body })) => body,
            other => panic!("expected an answer, got {other:?}"),
        }
    }

    const fn wire_vec3(x: f64, y: f64, z: f64) -> v1::Vec3 {
        v1::Vec3 { x, y, z }
    }

    fn view() -> EntityView {
        EntityView {
            id: ZOMBIE,
            uuid: Uuid::from_u128(7),
            kind: "minecraft:zombie".into(),
            position: Vec3::new(4.5, 64.0, -2.5),
        }
    }

    fn snapshot() -> v1::EntitySnapshot {
        v1::EntitySnapshot {
            entity_id: 42,
            uuid: "00000000-0000-0000-0000-000000000007".into(),
            kind: "minecraft:zombie".into(),
            position: Some(wire_vec3(4.5, 64.0, -2.5)),
        }
    }

    #[test]
    fn entity_orders_questions_and_answers_convert() {
        assert_eq!(
            entity(Some(entity_order::Order::Teleport(wire_vec3(
                1.0, 65.0, 2.0
            ))))
            .unwrap(),
            FromPlugin::Order(Order::Entity {
                entity: ZOMBIE,
                order: EntityOrder::Teleport(Vec3::new(1.0, 65.0, 2.0)),
            })
        );
        assert_eq!(
            entity(Some(entity_order::Order::Damage(3.5))).unwrap(),
            FromPlugin::Order(Order::Entity {
                entity: ZOMBIE,
                order: EntityOrder::Damage(3.5),
            })
        );
        assert!(matches!(entity(None), Err(Malformed::Empty)));

        let appearance = [
            (
                entity_order::Order::SetCustomName("§aGuide".into()),
                EntityOrder::SetCustomName("§aGuide".into()),
            ),
            (
                entity_order::Order::ClearCustomName(v1::Empty {}),
                EntityOrder::ClearCustomName,
            ),
            (
                entity_order::Order::SetNameVisible(true),
                EntityOrder::SetNameVisible(true),
            ),
            (
                entity_order::Order::SetNoAi(true),
                EntityOrder::SetNoAi(true),
            ),
            (
                entity_order::Order::SetInvulnerable(true),
                EntityOrder::SetInvulnerable(true),
            ),
            (
                entity_order::Order::SetSilent(false),
                EntityOrder::SetSilent(false),
            ),
        ];
        for (wire, order) in appearance {
            assert_eq!(
                entity(Some(wire)).unwrap(),
                FromPlugin::Order(Order::Entity {
                    entity: ZOMBIE,
                    order,
                })
            );
        }

        assert_eq!(
            ask(question::Question::Entities(v1::EntityQuery {
                near: Some(v1::Near {
                    position: Some(wire_vec3(4.5, 64.0, 4.5)),
                    radius: 2.0,
                }),
                kind: None,
            }))
            .unwrap(),
            asked(Question::Entities {
                near: Some((Vec3::new(4.5, 64.0, 4.5), 2.0)),
                kind: None,
            })
        );
        assert_eq!(
            ask(question::Question::Entities(v1::EntityQuery {
                near: None,
                kind: Some("minecraft:cow".into()),
            }))
            .unwrap(),
            asked(Question::Entities {
                near: None,
                kind: Some("minecraft:cow".into()),
            })
        );
        assert!(matches!(
            ask(question::Question::Entities(v1::EntityQuery {
                near: Some(v1::Near {
                    position: None,
                    radius: 2.0,
                }),
                kind: None,
            })),
            Err(Malformed::Empty)
        ));
        assert_eq!(
            ask(question::Question::Entity(42)).unwrap(),
            asked(Question::Entity(ZOMBIE))
        );
        assert_eq!(
            ask(question::Question::Spawn(v1::Spawn {
                kind: "minecraft:zombie".into(),
                position: Some(wire_vec3(4.5, 64.0, 4.5)),
            }))
            .unwrap(),
            asked(Question::Spawn {
                kind: "minecraft:zombie".into(),
                position: Vec3::new(4.5, 64.0, 4.5),
            })
        );
        assert_eq!(
            ask(question::Question::DropItem(v1::DropItem {
                position: Some(wire_vec3(2.5, 65.0, 2.5)),
                item: Some(v1::Item {
                    key: "minecraft:diamond".into(),
                    count: 3,
                    ..v1::Item::default()
                }),
            }))
            .unwrap(),
            asked(Question::DropItem {
                position: Vec3::new(2.5, 65.0, 2.5),
                item: ItemSpec::of("minecraft:diamond", 3),
            })
        );
        assert!(matches!(
            ask(question::Question::DropItem(v1::DropItem {
                position: Some(wire_vec3(2.5, 65.0, 2.5)),
                item: None,
            })),
            Err(Malformed::Empty)
        ));

        assert_eq!(
            answered(Answer::Spawned(view())),
            Some(answer::Body::Spawned(snapshot()))
        );
        assert_eq!(
            answered(Answer::Entities(vec![view()])),
            Some(answer::Body::Entities(v1::EntityList {
                entities: vec![snapshot()],
            }))
        );
        assert_eq!(
            answered(Answer::Refused("Unable to summon entity".into())),
            Some(answer::Body::Refused(v1::Refused {
                reason: "Unable to summon entity".into(),
            }))
        );
        assert_eq!(
            answered(Answer::Entity(EntityDetails {
                entity: view(),
                health: None,
                velocity: Vec3::new(0.0, -0.08, 0.0),
                on_ground: true,
                custom_name: Some("§aBob".into()),
                fire_ticks: -1,
            })),
            Some(answer::Body::Entity(v1::EntityState {
                entity: Some(snapshot()),
                health: None,
                velocity: Some(wire_vec3(0.0, -0.08, 0.0)),
                on_ground: true,
                custom_name: Some("§aBob".into()),
                fire_ticks: -1,
            }))
        );
    }

    #[test]
    fn an_interaction_converts_with_its_hand() {
        let player = PlayerSnapshot {
            id: PlayerId(Uuid::from_u128(1)),
            name: "Steve".into(),
            entity_id: 3,
            world: "minecraft:overworld".into(),
            position: Vec3::new(1.0, 64.0, 1.0),
            rotation: Rotation::new(0.0, 0.0),
            game_mode: GameMode::Survival,
        };
        let message = to_plugin(ToPlugin::Event {
            id: 5,
            handlers: vec![1],
            phase: Phase::Settled,
            cancelled: false,
            payload: EventPayload::EntityInteract {
                player,
                entity: view(),
                hand: Hand::Off,
                sneaking: true,
            },
        });
        let Some(to_plugin::Body::Event(v1::Event {
            payload: Some(event::Payload::EntityInteract(interact)),
            ..
        })) = message.body
        else {
            panic!("expected an interaction");
        };
        assert_eq!(interact.entity, Some(snapshot()));
        assert_eq!(interact.hand, v1::Hand::Off as i32);
        assert!(interact.sneaking);
        assert_eq!(interact.player.map(|p| p.name), Some("Steve".to_owned()));
    }
}
