mod error;
mod event;
mod link;
mod listener;
mod player;
mod runtime;
mod server;

pub use error::Error;
pub use event::{PlayerJoin, PlayerLeave, PlayerMove};
pub use listener::{Access, Call, EventKind, HandlerSpec, Listeners, Outcome, Plugin, Priority};
pub use player::{GameMode, Player, PlayerState, Position};
pub use rspigot_plugin_macros::{Plugin, listeners};
pub use runtime::{run, run_with};
pub use server::{Players, Server};
pub use uuid::Uuid;

pub mod prelude {
    pub use crate::{
        Error, GameMode, Listeners, Player, PlayerJoin, PlayerLeave, PlayerMove, PlayerState,
        Plugin, Position, Priority, Server, listeners, run, run_with,
    };
}
