use crate::{EntityType, EntityTypeId, MobCategory};

pub struct Painting;

impl EntityType for Painting {
    const NAME: &'static str = "minecraft:painting";
    const ID: EntityTypeId = EntityTypeId(94);
    const CLASS: &'static str = "Painting";
    const CATEGORY: MobCategory = MobCategory::Misc;
    const WIDTH: f32 = 0.5;
    const HEIGHT: f32 = 0.5;
    const EYE_HEIGHT: f32 = 0.425;
    const FIRE_IMMUNE: bool = false;
    const SUMMONABLE: bool = true;
    const CAN_SPAWN_FAR_FROM_PLAYER: bool = true;
    const CLIENT_TRACKING_RANGE: u8 = 10;
    const UPDATE_INTERVAL: Option<u8> = None;
    const SPAWN_EGG: Option<u16> = None;
    const LIVING: bool = false;
    const DEFAULT_ATTRIBUTES: &'static [(&'static str, f64)] = &[];
    type Behavior = crate::family::Plain;
}
