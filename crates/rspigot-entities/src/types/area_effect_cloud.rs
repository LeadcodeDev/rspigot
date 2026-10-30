use crate::{CreatureSounds, EntityType, EntityTypeId, MobCategory};

pub struct AreaEffectCloud;

impl EntityType for AreaEffectCloud {
    const NAME: &'static str = "minecraft:area_effect_cloud";
    const ID: EntityTypeId = EntityTypeId(3);
    const CLASS: &'static str = "AreaEffectCloud";
    const CATEGORY: MobCategory = MobCategory::Misc;
    const WIDTH: f32 = 6.0;
    const HEIGHT: f32 = 0.5;
    const EYE_HEIGHT: f32 = 0.425;
    const FIRE_IMMUNE: bool = true;
    const SUMMONABLE: bool = true;
    const CAN_SERIALIZE: bool = true;
    const ALLOWED_IN_PEACEFUL: bool = true;
    const CAN_SPAWN_FAR_FROM_PLAYER: bool = true;
    const CLIENT_TRACKING_RANGE: u8 = 10;
    const UPDATE_INTERVAL: Option<u8> = None;
    const TRACK_DELTAS: bool = true;
    const SPAWN_EGG: Option<u16> = None;
    const LIVING: bool = false;
    const ANCESTORS: &'static [&'static str] = &["AreaEffectCloud", "Entity"];
    const DEFAULT_ATTRIBUTES: &'static [(&'static str, f64)] = &[];
    const SOUNDS: Option<CreatureSounds> = None;
    type Behavior = crate::family::Plain;
}
