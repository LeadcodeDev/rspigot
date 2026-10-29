use rspigot_domain::item::Inventory;
use rspigot_domain::player::{ClientSettings, GameMode, GameProfile, PlayerId};
use rspigot_domain::{EntityId, Rotation, Vec3};
use rustc_hash::FxHashSet;

use crate::events::Abilities;
use crate::streaming::ChunkStreamer;

pub struct Player<L> {
    pub id: PlayerId,
    pub profile: GameProfile,
    pub link: L,
    pub entity_id: EntityId,
    pub position: Vec3,
    pub rotation: Rotation,
    pub on_ground: bool,
    pub game_mode: GameMode,
    pub previous_game_mode: Option<GameMode>,
    pub inventory: Inventory,
    pub inventory_state: i32,
    pub settings: ClientSettings,
    pub streamer: ChunkStreamer,
    pub pending_teleport: Option<i32>,
    pub teleport_counter: i32,
    pub sneaking: bool,
    pub sprinting: bool,
    pub flying: bool,
    pub tracked: FxHashSet<EntityId>,
    pub loaded: bool,
    pub operator: bool,
}

impl<L> Player<L> {
    pub fn name(&self) -> &str {
        &self.profile.name
    }

    pub fn abilities(&self) -> Abilities {
        Abilities {
            invulnerable: matches!(self.game_mode, GameMode::Creative | GameMode::Spectator),
            flying: self.flying || self.game_mode == GameMode::Spectator,
            may_fly: self.game_mode.can_fly(),
            instant_build: self.game_mode == GameMode::Creative,
            fly_speed: 0.05,
            walk_speed: 0.1,
        }
    }

    pub fn next_teleport_id(&mut self) -> i32 {
        self.teleport_counter = self.teleport_counter.wrapping_add(1);
        self.pending_teleport = Some(self.teleport_counter);
        self.teleport_counter
    }

    pub fn next_inventory_state(&mut self) -> i32 {
        self.inventory_state = self.inventory_state.wrapping_add(1) & 0x7FFF;
        self.inventory_state
    }

    pub fn shared_flags(&self) -> i8 {
        let mut flags = 0i8;
        if self.sneaking {
            flags |= 0x02;
        }
        if self.sprinting {
            flags |= 0x08;
        }
        flags
    }

    pub fn eye_position(&self) -> Vec3 {
        let eye = if self.sneaking { 1.27 } else { 1.62 };
        Vec3::new(self.position.x, self.position.y + eye, self.position.z)
    }
}
