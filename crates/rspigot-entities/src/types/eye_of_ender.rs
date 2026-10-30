use crate::{EntityType, EntityTypeId, MobCategory};

pub struct EyeOfEnder;

impl EntityType for EyeOfEnder {
    const NAME: &'static str = "minecraft:eye_of_ender";
    const ID: EntityTypeId = EntityTypeId(51);
    const CLASS: &'static str = "EyeOfEnder";
    const CATEGORY: MobCategory = MobCategory::Misc;
    const WIDTH: f32 = 0.25;
    const HEIGHT: f32 = 0.25;
    const EYE_HEIGHT: f32 = 0.2125;
    const FIRE_IMMUNE: bool = false;
    const SUMMONABLE: bool = true;
    const CAN_SPAWN_FAR_FROM_PLAYER: bool = true;
    const CLIENT_TRACKING_RANGE: u8 = 4;
    const UPDATE_INTERVAL: Option<u8> = Some(4);
    const SPAWN_EGG: Option<u16> = None;
    const LIVING: bool = false;
    const DEFAULT_ATTRIBUTES: &'static [(&'static str, f64)] = &[];
    type Behavior = crate::family::Plain;
}
