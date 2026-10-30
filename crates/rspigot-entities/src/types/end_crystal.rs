use crate::{CreatureSounds, EntityType, EntityTypeId, MobCategory};

pub struct EndCrystal;

impl EntityType for EndCrystal {
    const NAME: &'static str = "minecraft:end_crystal";
    const ID: EntityTypeId = EntityTypeId(46);
    const CLASS: &'static str = "EndCrystal";
    const CATEGORY: MobCategory = MobCategory::Misc;
    const WIDTH: f32 = 2.0;
    const HEIGHT: f32 = 2.0;
    const EYE_HEIGHT: f32 = 1.7;
    const FIRE_IMMUNE: bool = true;
    const SUMMONABLE: bool = true;
    const CAN_SERIALIZE: bool = true;
    const ALLOWED_IN_PEACEFUL: bool = true;
    const CAN_SPAWN_FAR_FROM_PLAYER: bool = true;
    const CLIENT_TRACKING_RANGE: u8 = 16;
    const UPDATE_INTERVAL: Option<u8> = None;
    const TRACK_DELTAS: bool = false;
    const SPAWN_EGG: Option<u16> = None;
    const LIVING: bool = false;
    const ANCESTORS: &'static [&'static str] = &["EndCrystal", "Entity"];
    const DEFAULT_ATTRIBUTES: &'static [(&'static str, f64)] = &[];
    const SOUNDS: Option<CreatureSounds> = None;
    type Behavior = crate::family::Plain;
}
