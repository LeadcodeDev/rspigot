use crate::{EntityType, EntityTypeId, MobCategory};

pub struct SmallFireball;

impl EntityType for SmallFireball {
    const NAME: &'static str = "minecraft:small_fireball";
    const ID: EntityTypeId = EntityTypeId(121);
    const CLASS: &'static str = "SmallFireball";
    const CATEGORY: MobCategory = MobCategory::Misc;
    const WIDTH: f32 = 0.3125;
    const HEIGHT: f32 = 0.3125;
    const EYE_HEIGHT: f32 = 0.265625;
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
