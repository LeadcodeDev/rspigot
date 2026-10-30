use crate::{EntityType, EntityTypeId, MobCategory};

pub struct LightningBolt;

impl EntityType for LightningBolt {
    const NAME: &'static str = "minecraft:lightning_bolt";
    const ID: EntityTypeId = EntityTypeId(78);
    const CLASS: &'static str = "LightningBolt";
    const CATEGORY: MobCategory = MobCategory::Misc;
    const WIDTH: f32 = 0.0;
    const HEIGHT: f32 = 0.0;
    const EYE_HEIGHT: f32 = 0.0;
    const FIRE_IMMUNE: bool = false;
    const SUMMONABLE: bool = true;
    const CAN_SPAWN_FAR_FROM_PLAYER: bool = true;
    const CLIENT_TRACKING_RANGE: u8 = 16;
    const UPDATE_INTERVAL: Option<u8> = None;
    const SPAWN_EGG: Option<u16> = None;
    const LIVING: bool = false;
    const DEFAULT_ATTRIBUTES: &'static [(&'static str, f64)] = &[];
    type Behavior = crate::family::Plain;
}
