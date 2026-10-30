use crate::{EntityType, EntityTypeId, MobCategory};

pub struct Tnt;

impl EntityType for Tnt {
    const NAME: &'static str = "minecraft:tnt";
    const ID: EntityTypeId = EntityTypeId(136);
    const CLASS: &'static str = "PrimedTnt";
    const CATEGORY: MobCategory = MobCategory::Misc;
    const WIDTH: f32 = 0.98;
    const HEIGHT: f32 = 0.98;
    const EYE_HEIGHT: f32 = 0.15;
    const FIRE_IMMUNE: bool = true;
    const SUMMONABLE: bool = true;
    const CAN_SPAWN_FAR_FROM_PLAYER: bool = true;
    const CLIENT_TRACKING_RANGE: u8 = 10;
    const UPDATE_INTERVAL: Option<u8> = Some(10);
    const SPAWN_EGG: Option<u16> = None;
    const LIVING: bool = false;
    const DEFAULT_ATTRIBUTES: &'static [(&'static str, f64)] = &[];
    type Behavior = crate::family::Plain;
}
