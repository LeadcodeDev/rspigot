#[rustfmt::skip]
mod blocks;
pub mod family;
#[rustfmt::skip]
mod kind;
mod state;

pub use blocks::*;
pub use family::{
    BlockBehavior, Face, Gravity, Linked, LinkedProperty, PlaceContext, Placement,
    SCAFFOLDING_MAX_DISTANCE, UseContext, UseResult, scaffolding_settle, speleothem_thickness,
};
pub use kind::{BlockKind, STATE_COUNT};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Default)]
pub struct StateId(pub u16);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct BlockId(pub u16);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Property {
    pub name: &'static str,
    pub values: &'static [&'static str],
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PerState<T: 'static> {
    Uniform(T),
    Varies(&'static [T]),
}

impl<T: Copy> PerState<T> {
    pub const fn get(&self, offset: u16) -> T {
        match self {
            Self::Uniform(value) => *value,
            Self::Varies(values) => values[offset as usize],
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct StateFlags(pub u16);

impl StateFlags {
    pub const AIR: Self = Self(1);
    pub const BLOCKS_MOTION: Self = Self(1 << 1);
    pub const HAS_COLLISION: Self = Self(1 << 2);
    pub const CAN_OCCLUDE: Self = Self(1 << 3);
    pub const SOLID_RENDER: Self = Self(1 << 4);
    pub const PROPAGATES_SKYLIGHT: Self = Self(1 << 5);
    pub const REPLACEABLE: Self = Self(1 << 6);
    pub const WATER: Self = Self(1 << 7);
    pub const LAVA: Self = Self(1 << 8);
    pub const RANDOMLY_TICKS: Self = Self(1 << 9);
    pub const FULL_COLLISION_CUBE: Self = Self(1 << 10);

    pub const fn contains(self, other: Self) -> bool {
        self.0 & other.0 == other.0
    }

    pub const fn union(self, other: Self) -> Self {
        Self(self.0 | other.0)
    }

    pub const fn has_fluid(self) -> bool {
        self.0 & (Self::WATER.0 | Self::LAVA.0) != 0
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct BlockSettings {
    pub hardness: f32,
    pub resistance: f32,
    pub friction: f32,
    pub speed_factor: f32,
    pub jump_factor: f32,
    pub requires_correct_tool: bool,
    pub ignited_by_lava: bool,
}

pub trait Block: 'static {
    const NAME: &'static str;
    const ID: BlockId;
    const FIRST_STATE: StateId;
    const STATE_COUNT: u16;
    const DEFAULT_STATE: StateId;
    const PROPERTIES: &'static [Property];
    const ITEM: Option<u16>;
    const WALL_VARIANT: Option<BlockId>;
    const SETTINGS: BlockSettings;
    const LIGHT_EMISSION: PerState<u8>;
    const LIGHT_DAMPENING: PerState<u8>;
    const FLAGS: PerState<StateFlags>;
    type Behavior: BlockBehavior;
}
