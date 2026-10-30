mod aabb;
mod base;
mod data;
pub mod falling;
pub mod item;
pub mod living;
pub mod physics;
#[cfg(test)]
mod physics_tests;
pub mod save;
mod store;
pub mod tracker;
mod tracking_range;

use rspigot_nbt::Compound;

pub use aabb::{Aabb, Axis};
pub use base::{
    DATA_AIR_SUPPLY, DATA_CUSTOM_NAME, DATA_CUSTOM_NAME_VISIBLE, DATA_NO_GRAVITY, DATA_POSE,
    DATA_SHARED_FLAGS, DATA_SILENT, DATA_TICKS_FROZEN, EntityBase, FLAG_GLOWING, FLAG_INVISIBLE,
    FLAG_ON_FIRE, MAX_AIR_SUPPLY, RemovalReason,
};
pub use data::{Pose, SyncedData, SyncedValue};
pub use falling::FallingState;
pub use item::ItemState;
pub use living::{LivingState, MobState};
pub use physics::BlockView;
pub use store::{DuplicateUuid, Entities};
pub use tracker::EntityTracker;
pub use tracking_range::{TrackingRanges, tracking_range};

#[derive(Debug, Clone, PartialEq)]
pub enum EntityState {
    Item(ItemState),
    Falling(FallingState),
    Living(Box<LivingState>),
    Inert(Compound),
}

#[derive(Debug, Clone, PartialEq)]
pub struct Entity {
    pub base: EntityBase,
    pub state: EntityState,
    pub extra: Compound,
}

impl Entity {
    pub fn new(base: EntityBase, state: EntityState) -> Self {
        Self {
            base,
            state,
            extra: Compound::new(),
        }
    }

    pub fn is_simulated(&self) -> bool {
        !matches!(self.state, EntityState::Inert(_))
    }

    pub fn living(&self) -> Option<&LivingState> {
        match &self.state {
            EntityState::Living(living) => Some(living),
            _ => None,
        }
    }

    pub fn living_mut(&mut self) -> Option<&mut LivingState> {
        match &mut self.state {
            EntityState::Living(living) => Some(living),
            _ => None,
        }
    }

    pub fn add_entity_data(&self) -> i32 {
        match &self.state {
            EntityState::Falling(falling) => i32::from(falling.state.0),
            _ => 0,
        }
    }
}
