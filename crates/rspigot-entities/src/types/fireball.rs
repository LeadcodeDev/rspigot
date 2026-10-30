use crate::{EntityType, EntityTypeId, MobCategory};

pub struct Fireball;

impl EntityType for Fireball {
    const NAME: &'static str = "minecraft:fireball";
    const ID: EntityTypeId = EntityTypeId(53);
    const CLASS: &'static str = "LargeFireball";
    const CATEGORY: MobCategory = MobCategory::Misc;
    const WIDTH: f32 = 1.0;
    const HEIGHT: f32 = 1.0;
    const EYE_HEIGHT: f32 = 0.85;
    const FIRE_IMMUNE: bool = false;
    const SUMMONABLE: bool = true;
    const CAN_SPAWN_FAR_FROM_PLAYER: bool = true;
    const CLIENT_TRACKING_RANGE: u8 = 4;
    const UPDATE_INTERVAL: Option<u8> = Some(10);
    const SPAWN_EGG: Option<u16> = None;
    const LIVING: bool = false;
    const DEFAULT_ATTRIBUTES: &'static [(&'static str, f64)] = &[];
    type Behavior = crate::family::Plain;
}
