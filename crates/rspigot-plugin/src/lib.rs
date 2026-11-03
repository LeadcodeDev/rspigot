mod color;
mod effect;
mod error;
mod event;
mod inventory;
mod item;
#[rustfmt::skip]
pub mod keys;
mod link;
mod listener;
mod player;
mod runtime;
mod server;
mod text;
mod wire;

pub use color::Color;
pub use effect::Effect;
pub use error::Error;
pub use event::{PlayerJoin, PlayerLeave, PlayerMove};
pub use inventory::{ArmorSlot, Inventory, InventorySlot, InventoryState};
pub use item::ItemStack;
pub use keys::{BlockType, EffectType, Enchantment, EntityType, Material, Particle, Sound};
pub use listener::{Access, Call, EventKind, HandlerSpec, Listeners, Outcome, Plugin, Priority};
pub use player::{GameMode, Player, PlayerState, Position, Rotation, Vector};
pub use rspigot_plugin_macros::{Plugin, listeners};
pub use runtime::{run, run_with};
pub use server::{Players, Server};
pub use text::{ChatColor, Title};
pub use uuid::Uuid;

pub mod prelude {
    pub use crate::{
        ArmorSlot, BlockType, ChatColor, Color, Effect, EffectType, Enchantment, EntityType, Error,
        GameMode, Inventory, InventorySlot, InventoryState, ItemStack, Listeners, Material,
        Particle, Player, PlayerJoin, PlayerLeave, PlayerMove, PlayerState, Plugin, Position,
        Priority, Rotation, Server, Sound, Title, Vector, listeners, run, run_with,
    };
}
