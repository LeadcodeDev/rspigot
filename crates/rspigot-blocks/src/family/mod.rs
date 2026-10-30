mod door;
mod facing;
mod falling;
mod fence_gate;
mod pillar;
mod scaffolding;
mod slab;
mod speleothem;
mod stairs;
mod standing;
mod trapdoor;

pub use door::Door;
pub use facing::{Directional, FaceAttached, HorizontalAway, HorizontalToward};
pub use falling::{Anvil, ConcretePowder, Falling};
pub use fence_gate::FenceGate;
pub use pillar::Pillar;
pub use scaffolding::{SCAFFOLDING_MAX_DISTANCE, Scaffolding, scaffolding_settle};
pub use slab::Slab;
pub use speleothem::{Speleothem, speleothem_thickness};
pub use stairs::Stairs;
pub use standing::StandingWithWall;
pub use trapdoor::Trapdoor;

use crate::StateId;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Face {
    Down,
    Up,
    North,
    South,
    West,
    East,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Gravity {
    None,
    Falling,
    ConcretePowder,
    Anvil,
    Speleothem,
    Scaffolding,
}

impl Gravity {
    pub const fn falls(self) -> bool {
        matches!(self, Self::Falling | Self::ConcretePowder | Self::Anvil)
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PlaceContext {
    pub default_state: StateId,
    pub wall_variant: Option<StateId>,
    pub face: Face,
    pub yaw: f32,
    pub pitch: f32,
    pub cursor_y: f32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Linked {
    pub dy: i8,
    pub state: StateId,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Placement {
    pub state: StateId,
    pub linked: Option<Linked>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct UseContext {
    pub state: StateId,
    pub sneaking: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct UseResult {
    pub state: StateId,
    pub linked: Option<LinkedProperty>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LinkedProperty {
    pub dy: i8,
    pub property: &'static str,
    pub value: &'static str,
}

pub trait BlockBehavior {
    const GRAVITY: Gravity = Gravity::None;

    fn place(context: &PlaceContext) -> Option<Placement> {
        Some(Placement {
            state: context.default_state,
            linked: None,
        })
    }

    fn use_block(_context: &UseContext) -> Option<UseResult> {
        None
    }
}

pub struct Plain;

impl BlockBehavior for Plain {}

impl Face {
    pub const ALL: [Self; 6] = [
        Self::Down,
        Self::Up,
        Self::North,
        Self::South,
        Self::West,
        Self::East,
    ];
    pub const HORIZONTAL: [Self; 4] = [Self::North, Self::East, Self::South, Self::West];

    pub const fn opposite(self) -> Self {
        match self {
            Self::Down => Self::Up,
            Self::Up => Self::Down,
            Self::North => Self::South,
            Self::South => Self::North,
            Self::West => Self::East,
            Self::East => Self::West,
        }
    }

    pub const fn offset(self) -> (i32, i32, i32) {
        match self {
            Self::Down => (0, -1, 0),
            Self::Up => (0, 1, 0),
            Self::North => (0, 0, -1),
            Self::South => (0, 0, 1),
            Self::West => (-1, 0, 0),
            Self::East => (1, 0, 0),
        }
    }

    pub const fn name(self) -> &'static str {
        match self {
            Self::Down => "down",
            Self::Up => "up",
            Self::North => "north",
            Self::South => "south",
            Self::West => "west",
            Self::East => "east",
        }
    }

    pub const fn is_horizontal(self) -> bool {
        !matches!(self, Self::Down | Self::Up)
    }
}

pub(crate) fn horizontal_facing(yaw: f32) -> &'static str {
    match ((f64::from(yaw) / 90.0 + 0.5).floor() as i64).rem_euclid(4) {
        0 => "south",
        1 => "west",
        2 => "north",
        _ => "east",
    }
}

pub(crate) fn looking(yaw: f32, pitch: f32) -> &'static str {
    if pitch > 45.0 {
        "down"
    } else if pitch < -45.0 {
        "up"
    } else {
        horizontal_facing(yaw)
    }
}

pub(crate) fn clockwise(direction: &str) -> &'static str {
    match direction {
        "north" => "east",
        "east" => "south",
        "south" => "west",
        _ => "north",
    }
}

pub(crate) fn opposite(direction: &str) -> &'static str {
    match direction {
        "south" => "north",
        "north" => "south",
        "east" => "west",
        "west" => "east",
        "up" => "down",
        _ => "up",
    }
}

pub(crate) fn upper_half(context: &PlaceContext) -> bool {
    match context.face {
        Face::Down => true,
        Face::Up => false,
        _ => context.cursor_y > 0.5,
    }
}

pub(crate) fn with(state: StateId, property: &str, value: &str) -> StateId {
    state.with_property(property, value).unwrap_or(state)
}

pub(crate) fn placed(state: StateId) -> Option<Placement> {
    Some(Placement {
        state,
        linked: None,
    })
}

pub(crate) fn toggled(state: StateId, property: &'static str) -> Option<(StateId, &'static str)> {
    let next = if state.property(property)? == "true" {
        "false"
    } else {
        "true"
    };
    Some((state.with_property(property, next)?, next))
}

#[cfg(test)]
pub(crate) mod testing {
    use super::{Face, PlaceContext};
    use crate::{BlockKind, StateId};

    pub fn context(block: &str, face: Face, yaw: f32, pitch: f32, cursor_y: f32) -> PlaceContext {
        let kind = BlockKind::from_name(block).unwrap_or(BlockKind::Air);
        PlaceContext {
            default_state: kind.default_state(),
            wall_variant: kind.wall_variant().map(BlockKind::default_state),
            face,
            yaw,
            pitch,
            cursor_y,
        }
    }

    pub fn property(state: StateId, name: &str) -> Option<&'static str> {
        state.property(name)
    }
}
