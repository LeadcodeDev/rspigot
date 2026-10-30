use crate::{EntityType, EntityTypeId, MobCategory};

pub struct Marker;

impl EntityType for Marker {
    const NAME: &'static str = "minecraft:marker";
    const ID: EntityTypeId = EntityTypeId(85);
    const CLASS: &'static str = "Marker";
    const CATEGORY: MobCategory = MobCategory::Misc;
    const WIDTH: f32 = 0.0;
    const HEIGHT: f32 = 0.0;
    const EYE_HEIGHT: f32 = 0.0;
    const FIRE_IMMUNE: bool = false;
    const SUMMONABLE: bool = true;
    const CAN_SERIALIZE: bool = true;
    const ALLOWED_IN_PEACEFUL: bool = true;
    const CAN_SPAWN_FAR_FROM_PLAYER: bool = true;
    const CLIENT_TRACKING_RANGE: u8 = 0;
    const UPDATE_INTERVAL: Option<u8> = Some(3);
    const TRACK_DELTAS: bool = true;
    const SPAWN_EGG: Option<u16> = None;
    const LIVING: bool = false;
    const ANCESTORS: &'static [&'static str] = &["Marker", "Entity"];
    const DEFAULT_ATTRIBUTES: &'static [(&'static str, f64)] = &[];
    type Behavior = crate::family::Plain;
}
