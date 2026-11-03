mod block;
mod color;
mod custom_inventory;
mod effect;
mod entity;
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
mod target;
mod text;
mod wire;
mod world;

pub use block::{Block, BlockPos, BlockState, Face};
pub use color::Color;
pub use custom_inventory::{ClickKind, ClickedSlot, CustomInventory, CustomInventoryState};
pub use effect::Effect;
pub use entity::{Entities, Entity, EntityState};
pub use error::Error;
pub use event::{InventoryClick, InventoryClose, PlayerJoin, PlayerLeave, PlayerMove};
pub use inventory::{ArmorSlot, Inventory, InventorySlot, InventoryState};
pub use item::ItemStack;
pub use keys::{BlockType, EffectType, Enchantment, EntityType, Material, Particle, Sound};
pub use listener::{
    Access, Call, EventKind, HandlerSpec, Listeners, Outcome, Plugin, Priority, Verdict,
};
pub use player::{GameMode, Player, PlayerState, Position, Rotation, Vector};
pub use rspigot_plugin_macros::{Plugin, listeners};
pub use runtime::{run, run_with};
pub use server::{
    BanEntry, BanList, BanListKind, Bans, ByAddress, ByName, IpBanEntry, IpBans, ListedPlayer,
    NameList, NameListKind, NamedPlayers, OperatorKind, Operators, Players, Server, ServerState,
    Whitelist, WhitelistKind,
};
pub use target::{BlockTarget, Target};
pub use text::{ChatColor, Title};
pub use uuid::Uuid;
pub use world::{Difficulty, GameRule, Weather, World, WorldState};

pub mod prelude {
    pub use crate::{
        ArmorSlot, Block, BlockPos, BlockState, BlockTarget, BlockType, ChatColor, ClickKind,
        ClickedSlot, Color, CustomInventory, CustomInventoryState, Difficulty, Effect, EffectType,
        Enchantment, Entity, EntityState, EntityType, Error, Face, GameMode, GameRule, Inventory,
        InventoryClick, InventoryClose, InventorySlot, InventoryState, ItemStack, Listeners,
        Material, Particle, Player, PlayerJoin, PlayerLeave, PlayerMove, PlayerState, Plugin,
        Position, Priority, Rotation, Server, ServerState, Sound, Target, Title, Vector, Weather,
        World, WorldState, listeners, run, run_with,
    };
}
