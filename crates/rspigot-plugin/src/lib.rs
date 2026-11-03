mod color;
mod error;
mod event;
#[rustfmt::skip]
pub mod keys;
mod link;
mod listener;
mod player;
mod runtime;
mod server;

pub use color::Color;
pub use error::Error;
pub use event::{PlayerJoin, PlayerLeave, PlayerMove};
pub use keys::{BlockType, EffectType, Enchantment, EntityType, Material, Particle, Sound};
pub use listener::{Access, Call, EventKind, HandlerSpec, Listeners, Outcome, Plugin, Priority};
pub use player::{GameMode, Player, PlayerState, Position, Rotation};
pub use rspigot_plugin_macros::{Plugin, listeners};
pub use runtime::{run, run_with};
pub use server::{Players, Server};
pub use uuid::Uuid;

pub mod prelude {
    pub use crate::{
        BlockType, Color, EffectType, Enchantment, EntityType, Error, GameMode, Listeners,
        Material, Particle, Player, PlayerJoin, PlayerLeave, PlayerMove, PlayerState, Plugin,
        Position, Priority, Rotation, Server, Sound, listeners, run, run_with,
    };
}
