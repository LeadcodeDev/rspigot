use crate::{CreatureSounds, EntityType, EntityTypeId, MobCategory};

pub struct EvokerFangs;

impl EntityType for EvokerFangs {
    const NAME: &'static str = "minecraft:evoker_fangs";
    const ID: EntityTypeId = EntityTypeId(48);
    const CLASS: &'static str = "EvokerFangs";
    const CATEGORY: MobCategory = MobCategory::Misc;
    const WIDTH: f32 = 0.5;
    const HEIGHT: f32 = 0.8;
    const EYE_HEIGHT: f32 = 0.68;
    const FIRE_IMMUNE: bool = false;
    const SUMMONABLE: bool = true;
    const CAN_SERIALIZE: bool = true;
    const ALLOWED_IN_PEACEFUL: bool = true;
    const CAN_SPAWN_FAR_FROM_PLAYER: bool = true;
    const CLIENT_TRACKING_RANGE: u8 = 6;
    const UPDATE_INTERVAL: Option<u8> = Some(2);
    const TRACK_DELTAS: bool = false;
    const SPAWN_EGG: Option<u16> = None;
    const LIVING: bool = false;
    const ANCESTORS: &'static [&'static str] = &["EvokerFangs", "Entity"];
    const DEFAULT_ATTRIBUTES: &'static [(&'static str, f64)] = &[];
    const SOUNDS: Option<CreatureSounds> = None;
    type Behavior = crate::family::Plain;
}
