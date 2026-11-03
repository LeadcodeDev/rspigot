mod inventory;
mod player;
mod server;
mod shared;

use rspigot_core::plugins::{
    Access, Answer, EventKind, EventPayload, FromPlugin, HandlerSpec, Phase, PluginIntro, Priority,
    Question, ToPlugin,
};
use rspigot_plugin_protocol::v1::{self, answer, event, from_plugin, question, to_plugin};

use player::{wire_player, wire_state};
use shared::wire_vec;

#[derive(Debug, thiserror::Error)]
pub(crate) enum Malformed {
    #[error("handler {handler} has an unknown {field} value {value}")]
    Unknown {
        handler: u32,
        field: &'static str,
        value: i32,
    },
    #[error("`{0}` is not a player UUID")]
    Player(String),
    #[error("the message carries nothing")]
    Empty,
    #[error("the plugin already introduced itself")]
    SecondHello,
    #[error("`{0}` is not supported yet")]
    Unsupported(&'static str),
    #[error("unknown {field} value {value}")]
    Enum { field: &'static str, value: i32 },
}

pub(crate) fn intro(hello: v1::Hello) -> Result<PluginIntro, Malformed> {
    Ok(PluginIntro {
        name: hello.name,
        version: hello.version,
        handlers: hello
            .handlers
            .into_iter()
            .map(handler)
            .collect::<Result<_, _>>()?,
    })
}

fn handler(handler: v1::Handler) -> Result<HandlerSpec, Malformed> {
    let unknown = |field, value| Malformed::Unknown {
        handler: handler.id,
        field,
        value,
    };
    let event = match v1::EventKind::try_from(handler.event) {
        Ok(v1::EventKind::PlayerJoin) => EventKind::PlayerJoin,
        Ok(v1::EventKind::PlayerLeave) => EventKind::PlayerLeave,
        Ok(v1::EventKind::PlayerMove) => EventKind::PlayerMove,
        Ok(
            v1::EventKind::Unspecified
            | v1::EventKind::InventoryClick
            | v1::EventKind::InventoryClose,
        )
        | Err(_) => return Err(unknown("event", handler.event)),
    };
    let access = match v1::Access::try_from(handler.access) {
        Ok(v1::Access::Observe) => Access::Observe,
        Ok(v1::Access::Decide) => Access::Decide,
        Ok(v1::Access::Unspecified) | Err(_) => return Err(unknown("access", handler.access)),
    };
    let priority = match v1::Priority::try_from(handler.priority) {
        Ok(v1::Priority::Lowest) => Priority::Lowest,
        Ok(v1::Priority::Low) => Priority::Low,
        Ok(v1::Priority::Unspecified | v1::Priority::Normal) => Priority::Normal,
        Ok(v1::Priority::High) => Priority::High,
        Ok(v1::Priority::Highest) => Priority::Highest,
        Err(_) => return Err(unknown("priority", handler.priority)),
    };
    Ok(HandlerSpec {
        id: handler.id,
        event,
        access,
        priority,
        ignore_cancelled: handler.ignore_cancelled,
    })
}

pub(crate) fn from_plugin(message: v1::FromPlugin) -> Result<FromPlugin, Malformed> {
    Ok(match message.body.ok_or(Malformed::Empty)? {
        from_plugin::Body::Hello(_) => return Err(Malformed::SecondHello),
        from_plugin::Body::Decision(decision) => FromPlugin::Decision {
            event: decision.event,
            cancel: decision.cancel,
        },
        from_plugin::Body::Player(order) => FromPlugin::Order(player::order(order)?),
        from_plugin::Body::Server(order) => FromPlugin::Order(server::order(order)?),
        from_plugin::Body::Inventory(order) => FromPlugin::Order(inventory::order(order)?),
        from_plugin::Body::World(_) => return Err(Malformed::Unsupported("world order")),
        from_plugin::Body::Block(_) => return Err(Malformed::Unsupported("block order")),
        from_plugin::Body::Entity(_) => return Err(Malformed::Unsupported("entity order")),
        from_plugin::Body::CustomInventory(_) => {
            return Err(Malformed::Unsupported("custom inventory order"));
        }
        from_plugin::Body::Question(asked) => question(asked)?,
    })
}

fn question(asked: v1::Question) -> Result<FromPlugin, Malformed> {
    let question = match asked.question.ok_or(Malformed::Empty)? {
        question::Question::Player(player) => Question::Player(player::id(player)?),
        question::Question::Players(_) => Question::Players,
        question::Question::Inventory(player) => Question::Inventory(player::id(player)?),
        _ => return Err(Malformed::Unsupported("question")),
    };
    Ok(FromPlugin::Question {
        request: asked.request,
        question,
    })
}

pub(crate) fn to_plugin(message: ToPlugin) -> v1::ToPlugin {
    let body = match message {
        ToPlugin::Accepted => to_plugin::Body::Accepted(v1::Accepted {}),
        ToPlugin::Event {
            id,
            handlers,
            phase,
            cancelled,
            payload,
        } => to_plugin::Body::Event(v1::Event {
            id,
            handlers,
            phase: wire_phase(phase) as i32,
            cancelled,
            payload: Some(wire_payload(payload)),
        }),
        ToPlugin::Answer { request, answer } => to_plugin::Body::Answer(v1::Answer {
            request,
            body: Some(wire_answer(answer)),
        }),
        ToPlugin::Warning(message) => to_plugin::Body::Warning(v1::Warning { message }),
        ToPlugin::Replaced => to_plugin::Body::Replaced(v1::Replaced {}),
    };
    v1::ToPlugin { body: Some(body) }
}

const fn wire_phase(phase: Phase) -> v1::Phase {
    match phase {
        Phase::Decide => v1::Phase::Decide,
        Phase::Settled => v1::Phase::Settled,
        Phase::RolledBack => v1::Phase::RolledBack,
    }
}

fn wire_payload(payload: EventPayload) -> event::Payload {
    match payload {
        EventPayload::PlayerJoin { player } => event::Payload::PlayerJoin(v1::PlayerJoin {
            player: Some(wire_player(player)),
        }),
        EventPayload::PlayerLeave { player } => event::Payload::PlayerLeave(v1::PlayerLeave {
            player: Some(wire_player(player)),
        }),
        EventPayload::PlayerMove { player, from, to } => {
            event::Payload::PlayerMove(v1::PlayerMove {
                player: Some(wire_player(player)),
                from: Some(wire_vec(from)),
                to: Some(wire_vec(to)),
            })
        }
    }
}

fn wire_answer(answer: Answer) -> answer::Body {
    match answer {
        Answer::Player(state) => answer::Body::Player(wire_state(state)),
        Answer::Players(players) => answer::Body::Players(v1::PlayerList {
            players: players.into_iter().map(wire_player).collect(),
        }),
        Answer::Inventory(state) => answer::Body::Inventory(inventory::wire_state(state)),
        Answer::NotFound => answer::Body::NotFound(v1::NotFound {}),
    }
}

#[cfg(test)]
mod tests {
    use rspigot_core::plugins::{Order, PlayerOrder, PlayerSnapshot};
    use rspigot_domain::player::{GameMode, PlayerId};
    use rspigot_domain::{Rotation, Vec3};
    use rspigot_plugin_protocol::v1::{player_order, world_order};
    use uuid::Uuid;

    use super::*;

    const STEVE: &str = "00000000-0000-0000-0000-000000000001";

    fn body(body: from_plugin::Body) -> v1::FromPlugin {
        v1::FromPlugin { body: Some(body) }
    }

    #[test]
    fn a_player_order_and_the_two_questions_keep_their_shape() {
        let steve = PlayerId(Uuid::from_u128(1));
        let said = body(from_plugin::Body::Player(v1::PlayerOrder {
            player: STEVE.into(),
            order: Some(player_order::Order::SendMessage("hi".into())),
        }));
        assert_eq!(
            from_plugin(said).unwrap(),
            FromPlugin::Order(Order::Player {
                player: steve,
                order: PlayerOrder::SendMessage("hi".into())
            })
        );
        let asked = body(from_plugin::Body::Question(v1::Question {
            request: 7,
            question: Some(question::Question::Player(STEVE.into())),
        }));
        assert_eq!(
            from_plugin(asked).unwrap(),
            FromPlugin::Question {
                request: 7,
                question: Question::Player(steve)
            }
        );
        let all = body(from_plugin::Body::Question(v1::Question {
            request: 8,
            question: Some(question::Question::Players(v1::Empty {})),
        }));
        assert_eq!(
            from_plugin(all).unwrap(),
            FromPlugin::Question {
                request: 8,
                question: Question::Players
            }
        );
    }

    #[test]
    fn an_order_or_question_not_supported_yet_is_dropped() {
        let hide = body(from_plugin::Body::Player(v1::PlayerOrder {
            player: STEVE.into(),
            order: Some(player_order::Order::HidePlayer(STEVE.into())),
        }));
        assert!(matches!(
            from_plugin(hide),
            Err(Malformed::Unsupported("player order"))
        ));
        let save = body(from_plugin::Body::World(v1::WorldOrder {
            order: Some(world_order::Order::Save(v1::Empty {})),
        }));
        assert!(matches!(
            from_plugin(save),
            Err(Malformed::Unsupported("world order"))
        ));
        let server = body(from_plugin::Body::Question(v1::Question {
            request: 1,
            question: Some(question::Question::Server(v1::Empty {})),
        }));
        assert!(matches!(
            from_plugin(server),
            Err(Malformed::Unsupported("question"))
        ));
    }

    #[test]
    fn a_snapshot_carries_its_rotation() {
        let snapshot = PlayerSnapshot {
            id: PlayerId(Uuid::from_u128(1)),
            name: "Steve".to_owned(),
            entity_id: 12,
            world: "minecraft:overworld".to_owned(),
            position: Vec3::new(1.0, 64.0, -3.5),
            rotation: Rotation::new(90.0, -30.0),
            game_mode: GameMode::Creative,
        };
        assert_eq!(
            wire_player(snapshot).rotation,
            Some(v1::Rotation {
                yaw: 90.0,
                pitch: -30.0
            })
        );
    }
}
