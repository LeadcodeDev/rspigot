use crate::{EntityType, EntityTypeId, MobCategory};

pub struct AcaciaBoat;

impl EntityType for AcaciaBoat {
    const NAME: &'static str = "minecraft:acacia_boat";
    const ID: EntityTypeId = EntityTypeId(0);
    const CLASS: &'static str = "Boat";
    const CATEGORY: MobCategory = MobCategory::Misc;
    const WIDTH: f32 = 1.375;
    const HEIGHT: f32 = 0.5625;
    const EYE_HEIGHT: f32 = 0.5625;
    const FIRE_IMMUNE: bool = false;
    const SUMMONABLE: bool = true;
    const CAN_SPAWN_FAR_FROM_PLAYER: bool = true;
    const CLIENT_TRACKING_RANGE: u8 = 10;
    const UPDATE_INTERVAL: Option<u8> = Some(3);
    const SPAWN_EGG: Option<u16> = None;
    const LIVING: bool = false;
    const DEFAULT_ATTRIBUTES: &'static [(&'static str, f64)] = &[];
    type Behavior = crate::family::Plain;
}
