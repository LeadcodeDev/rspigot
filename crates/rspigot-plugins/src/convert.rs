use rspigot_core::plugins::{
    Access, Answer, EventKind, EventPayload, FromPlugin, HandlerSpec, Phase, PlayerSnapshot,
    PlayerState, PluginIntro, Priority, ToPlugin,
};
use rspigot_domain::Vec3;
use rspigot_domain::player::{GameMode, PlayerId};
use rspigot_plugin_protocol::v1::{self, answer, event, from_plugin, to_plugin};
use uuid::Uuid;

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
        Ok(v1::EventKind::Unspecified) | Err(_) => return Err(unknown("event", handler.event)),
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
        from_plugin::Body::SendMessage(order) => FromPlugin::SendMessage {
            player: player_id(order.player)?,
            text: order.text,
        },
        from_plugin::Body::ResolvePlayer(question) => FromPlugin::ResolvePlayer {
            request: question.request,
            player: player_id(question.player)?,
        },
        from_plugin::Body::FetchPlayers(question) => FromPlugin::FetchPlayers {
            request: question.request,
        },
    })
}

fn player_id(raw: String) -> Result<PlayerId, Malformed> {
    Uuid::parse_str(&raw)
        .map(PlayerId)
        .map_err(|_| Malformed::Player(raw))
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
        Answer::NotFound => answer::Body::NotFound(v1::NotFound {}),
    }
}

fn wire_state(state: PlayerState) -> v1::PlayerState {
    v1::PlayerState {
        player: Some(wire_player(state.player)),
        health: state.health,
        food: state.food,
        level: state.level,
    }
}

fn wire_player(player: PlayerSnapshot) -> v1::PlayerSnapshot {
    v1::PlayerSnapshot {
        uuid: player.id.0.hyphenated().to_string(),
        name: player.name,
        entity_id: player.entity_id,
        world: player.world,
        position: Some(wire_vec(player.position)),
        game_mode: wire_game_mode(player.game_mode) as i32,
    }
}

const fn wire_vec(vec: Vec3) -> v1::Vec3 {
    v1::Vec3 {
        x: vec.x,
        y: vec.y,
        z: vec.z,
    }
}

const fn wire_game_mode(mode: GameMode) -> v1::GameMode {
    match mode {
        GameMode::Survival => v1::GameMode::Survival,
        GameMode::Creative => v1::GameMode::Creative,
        GameMode::Adventure => v1::GameMode::Adventure,
        GameMode::Spectator => v1::GameMode::Spectator,
    }
}
