use crate::{EntityType, EntityTypeId, MobCategory};

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
    const CAN_SPAWN_FAR_FROM_PLAYER: bool = true;
    const CLIENT_TRACKING_RANGE: u8 = 10;
    const UPDATE_INTERVAL: Option<u8> = None;
    const SPAWN_EGG: Option<u16> = None;
    const LIVING: bool = false;
    const DEFAULT_ATTRIBUTES: &'static [(&'static str, f64)] = &[];
    type Behavior = crate::family::Plain;
}
