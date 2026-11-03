mod block;
mod decisions;
mod entity;
mod inventory;
mod items;
mod menus;
mod orders;
mod player;
mod registry;
mod resolve;
mod server;
mod targets;
mod visibility;
mod world;

use std::net::IpAddr;

use rspigot_domain::player::{GameMode, PlayerId};
use rspigot_domain::world::Difficulty;
use rspigot_domain::{BlockPos, EntityId, Rotation, Vec3};
use rustc_hash::FxHashSet;
use uuid::Uuid;

use crate::admin::{BanEntry, IpBanEntry, ListEntry};
use crate::events::Direction;

pub(crate) use decisions::{Decisions, Held};
pub use items::ItemSpec;
pub use menus::MAX_PLUGIN_INVENTORIES;
pub(crate) use menus::{PluginInventories, PluginMenu};
pub use orders::{
    BlockOrder, CustomInventoryOrder, EffectSpec, EntityOrder, GameRule, InventoryOrder, Order,
    ParticleDataSpec, ParticleSpec, PlayerOrder, ServerOrder, SoundSpec, TitleSpec, WorldOrder,
};
pub(crate) use registry::Registry;
pub use targets::MAX_TARGET_DISTANCE;
pub(crate) use visibility::Visibility;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum EventKind {
    PlayerJoin,
    PlayerLeave,
    PlayerMove,
    InventoryClick,
    InventoryClose,
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
    pub rotation: Rotation,
    pub game_mode: GameMode,
}

#[derive(Debug, Clone, PartialEq)]
pub struct PlayerState {
    pub player: PlayerSnapshot,
    pub health: f32,
    pub max_health: f32,
    pub absorption: f32,
    pub food: i32,
    pub saturation: f32,
    pub exhaustion: f32,
    pub level: i32,
    pub exp: f32,
    pub total_exp: i32,
    pub operator: bool,
    pub allow_flight: bool,
    pub flying: bool,
    pub fly_speed: f32,
    pub walk_speed: f32,
    pub sneaking: bool,
    pub sprinting: bool,
    pub on_ground: bool,
    pub fire_ticks: i32,
    pub dead: bool,
    pub effects: Vec<EffectSpec>,
    pub ping: i32,
    pub locale: String,
    pub address: IpAddr,
    pub hidden: Vec<PlayerId>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InventoryState {
    pub slots: Vec<Option<ItemSpec>>,
    pub held_slot: u8,
}

#[derive(Debug, Clone, PartialEq)]
pub struct CustomInventoryState {
    pub items: Vec<Option<ItemSpec>>,
    pub viewers: Vec<PlayerSnapshot>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ServerState {
    pub max_players: u32,
    pub online: u32,
    pub tps: [f64; 3],
    pub whitelist_enabled: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WorldState {
    pub name: String,
    pub uuid: Option<Uuid>,
    pub seed: i64,
    pub day_time: i64,
    pub game_time: i64,
    pub raining: bool,
    pub thundering: bool,
    pub difficulty: Difficulty,
    pub spawn: BlockPos,
    pub pvp: bool,
    pub rules: Vec<(GameRule, bool)>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BlockState {
    pub kind: String,
    pub properties: Vec<(String, String)>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct EntityView {
    pub id: EntityId,
    pub uuid: Uuid,
    pub kind: String,
    pub position: Vec3,
}

#[derive(Debug, Clone, PartialEq)]
pub struct EntityDetails {
    pub entity: EntityView,
    pub health: Option<f32>,
    pub velocity: Vec3,
    pub on_ground: bool,
    pub custom_name: Option<String>,
    pub fire_ticks: i32,
}

#[derive(Debug, Clone, PartialEq)]
pub enum TargetHit {
    Block {
        pos: BlockPos,
        face: Direction,
        kind: String,
    },
    Player(PlayerSnapshot),
    Entity(EntityView),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ClickedSlot {
    Menu(u16),
    Player(i32),
    Outside,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ClickKind {
    Left,
    Right,
    ShiftLeft,
    ShiftRight,
    NumberKey(u8),
    SwapOffhand,
    Drop,
    ControlDrop,
    DoubleClick,
    Middle,
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
    InventoryClick {
        player: PlayerSnapshot,
        inventory: u64,
        slot: ClickedSlot,
        kind: ClickKind,
        item: Option<ItemSpec>,
    },
    InventoryClose {
        player: PlayerSnapshot,
        inventory: u64,
        by_player: bool,
    },
}

impl EventPayload {
    pub const fn kind(&self) -> EventKind {
        match self {
            Self::PlayerJoin { .. } => EventKind::PlayerJoin,
            Self::PlayerLeave { .. } => EventKind::PlayerLeave,
            Self::PlayerMove { .. } => EventKind::PlayerMove,
            Self::InventoryClick { .. } => EventKind::InventoryClick,
            Self::InventoryClose { .. } => EventKind::InventoryClose,
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
    Inventory(InventoryState),
    Server(ServerState),
    Names(Vec<ListEntry>),
    Bans(Vec<BanEntry>),
    IpBans(Vec<IpBanEntry>),
    World(WorldState),
    Block(BlockState),
    Entities(Vec<EntityView>),
    Entity(EntityDetails),
    Spawned(EntityView),
    Target(Option<TargetHit>),
    CustomInventory(CustomInventoryState),
    Refused(String),
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
    Order(Order),
    Question { request: u64, question: Question },
}

#[derive(Debug, Clone, PartialEq)]
pub enum Question {
    Player(PlayerId),
    Players,
    PlayerNamed(String),
    Inventory(PlayerId),
    TargetBlock {
        player: PlayerId,
        max_distance: f64,
    },
    TargetEntity {
        player: PlayerId,
        max_distance: f64,
    },
    Server,
    Whitelist,
    Operators,
    Bans,
    IpBans,
    World,
    Block(BlockPos),
    Entities {
        near: Option<(Vec3, f64)>,
        kind: Option<String>,
    },
    Entity(EntityId),
    Spawn {
        kind: String,
        position: Vec3,
    },
    DropItem {
        position: Vec3,
        item: ItemSpec,
    },
    CustomInventory(u64),
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
        assert!(!EventKind::InventoryClick.decidable());
        assert!(!EventKind::InventoryClose.decidable());
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
