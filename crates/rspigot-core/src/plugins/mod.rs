mod decisions;
mod registry;
mod server;

use rspigot_domain::Vec3;
use rspigot_domain::player::{GameMode, PlayerId};
use rustc_hash::FxHashSet;

pub(crate) use decisions::Decisions;
pub(crate) use registry::Registry;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum EventKind {
    PlayerJoin,
    PlayerLeave,
    PlayerMove,
}

impl EventKind {
    pub const fn decidable(self) -> bool {
        matches!(self, Self::PlayerMove)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Access {
    Observe,
    Decide,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Default)]
pub enum Priority {
    Lowest,
    Low,
    #[default]
    Normal,
    High,
    Highest,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct HandlerSpec {
    pub id: u32,
    pub event: EventKind,
    pub access: Access,
    pub priority: Priority,
    pub ignore_cancelled: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PluginIntro {
    pub name: String,
    pub version: String,
    pub handlers: Vec<HandlerSpec>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
pub enum IntroError {
    #[error("the plugin name is empty")]
    EmptyName,
    #[error("handler {0} is declared more than once")]
    DuplicateHandler(u32),
    #[error("handler {0} decides an event that cannot be decided")]
    Undecidable(u32),
    #[error("handler {0} decides and ignores cancelled events, which only an observer can do")]
    ObserverOnly(u32),
}

impl PluginIntro {
    pub fn validate(&self) -> Result<(), IntroError> {
        if self.name.is_empty() {
            return Err(IntroError::EmptyName);
        }
        let mut seen = FxHashSet::default();
        for handler in &self.handlers {
            if !seen.insert(handler.id) {
                return Err(IntroError::DuplicateHandler(handler.id));
            }
            if handler.access == Access::Decide {
                if !handler.event.decidable() {
                    return Err(IntroError::Undecidable(handler.id));
                }
                if handler.ignore_cancelled {
                    return Err(IntroError::ObserverOnly(handler.id));
                }
            }
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PlayerSnapshot {
    pub id: PlayerId,
    pub name: String,
    pub entity_id: i32,
    pub world: String,
    pub position: Vec3,
    pub game_mode: GameMode,
}

#[derive(Debug, Clone, PartialEq)]
pub struct PlayerState {
    pub player: PlayerSnapshot,
    pub health: f32,
    pub food: i32,
    pub level: i32,
}

#[derive(Debug, Clone, PartialEq)]
pub enum EventPayload {
    PlayerJoin {
        player: PlayerSnapshot,
    },
    PlayerLeave {
        player: PlayerSnapshot,
    },
    PlayerMove {
        player: PlayerSnapshot,
        from: Vec3,
        to: Vec3,
    },
}

impl EventPayload {
    pub const fn kind(&self) -> EventKind {
        match self {
            Self::PlayerJoin { .. } => EventKind::PlayerJoin,
            Self::PlayerLeave { .. } => EventKind::PlayerLeave,
            Self::PlayerMove { .. } => EventKind::PlayerMove,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Phase {
    Decide,
    Settled,
    RolledBack,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Answer {
    Player(PlayerState),
    Players(Vec<PlayerSnapshot>),
    NotFound,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ToPlugin {
    Accepted,
    Event {
        id: u64,
        handlers: Vec<u32>,
        phase: Phase,
        cancelled: bool,
        payload: EventPayload,
    },
    Answer {
        request: u64,
        answer: Answer,
    },
    Warning(String),
    Replaced,
}

#[derive(Debug, Clone, PartialEq)]
pub enum FromPlugin {
    Decision { event: u64, cancel: bool },
    SendMessage { player: PlayerId, text: String },
    ResolvePlayer { request: u64, player: PlayerId },
    FetchPlayers { request: u64 },
}

#[cfg(test)]
mod tests {
    use super::*;

    fn handler(id: u32, event: EventKind, access: Access) -> HandlerSpec {
        HandlerSpec {
            id,
            event,
            access,
            priority: Priority::Normal,
            ignore_cancelled: false,
        }
    }

    fn intro(handlers: Vec<HandlerSpec>) -> PluginIntro {
        PluginIntro {
            name: "welcome".to_owned(),
            version: "1.0.0".to_owned(),
            handlers,
        }
    }

    #[test]
    fn an_introduction_with_duplicate_ids_or_an_undecidable_decision_is_invalid() {
        let observer = handler(1, EventKind::PlayerJoin, Access::Observe);
        let decider = handler(2, EventKind::PlayerMove, Access::Decide);
        assert_eq!(intro(vec![observer, decider]).validate(), Ok(()));
        assert_eq!(intro(Vec::new()).validate(), Ok(()));

        let twin = handler(1, EventKind::PlayerLeave, Access::Observe);
        assert_eq!(
            intro(vec![observer, twin]).validate(),
            Err(IntroError::DuplicateHandler(1))
        );

        let join_decider = handler(3, EventKind::PlayerJoin, Access::Decide);
        assert_eq!(
            intro(vec![observer, join_decider]).validate(),
            Err(IntroError::Undecidable(3))
        );

        let mut filtered_decider = decider;
        filtered_decider.ignore_cancelled = true;
        assert_eq!(
            intro(vec![observer, filtered_decider]).validate(),
            Err(IntroError::ObserverOnly(2))
        );

        let mut filtered_observer = observer;
        filtered_observer.ignore_cancelled = true;
        assert_eq!(intro(vec![filtered_observer]).validate(), Ok(()));

        let mut nameless = intro(vec![observer]);
        nameless.name.clear();
        assert_eq!(nameless.validate(), Err(IntroError::EmptyName));
    }

    #[test]
    fn only_a_move_can_be_decided() {
        assert!(EventKind::PlayerMove.decidable());
        assert!(!EventKind::PlayerJoin.decidable());
        assert!(!EventKind::PlayerLeave.decidable());
    }

    #[test]
    fn priorities_run_from_lowest_to_highest_and_default_to_normal() {
        assert!(Priority::Lowest < Priority::Low);
        assert!(Priority::Low < Priority::Normal);
        assert!(Priority::Normal < Priority::High);
        assert!(Priority::High < Priority::Highest);
        assert_eq!(Priority::default(), Priority::Normal);
    }
}
