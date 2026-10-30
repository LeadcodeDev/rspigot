#[rustfmt::skip]
mod categories;
mod category;
pub mod family;
#[rustfmt::skip]
mod kind;
mod lookup;
#[rustfmt::skip]
mod types;

pub use categories::MobCategory;
pub use family::EntityBehavior;
pub use kind::EntityKind;
pub use types::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Default)]
pub struct EntityTypeId(pub u16);

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct EntityDimensions {
    pub width: f32,
    pub height: f32,
    pub eye_height: f32,
}

pub trait EntityType: 'static {
    const NAME: &'static str;
    const ID: EntityTypeId;
    const CLASS: &'static str;
    const CATEGORY: MobCategory;
    const WIDTH: f32;
    const HEIGHT: f32;
    const EYE_HEIGHT: f32;
    const FIRE_IMMUNE: bool;
    const SUMMONABLE: bool;
    const CAN_SERIALIZE: bool;
    const ALLOWED_IN_PEACEFUL: bool;
    const CAN_SPAWN_FAR_FROM_PLAYER: bool;
    const CLIENT_TRACKING_RANGE: u8;
    const UPDATE_INTERVAL: Option<u8>;
    const TRACK_DELTAS: bool;
    const SPAWN_EGG: Option<u16>;
    const LIVING: bool;
    const ANCESTORS: &'static [&'static str];
    const DEFAULT_ATTRIBUTES: &'static [(&'static str, f64)];
    type Behavior: EntityBehavior;
}
