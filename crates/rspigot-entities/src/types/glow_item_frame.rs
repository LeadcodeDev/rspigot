use crate::{EntityType, EntityTypeId, MobCategory};

pub struct GlowItemFrame;

impl EntityType for GlowItemFrame {
    const NAME: &'static str = "minecraft:glow_item_frame";
    const ID: EntityTypeId = EntityTypeId(61);
    const CLASS: &'static str = "GlowItemFrame";
    const CATEGORY: MobCategory = MobCategory::Misc;
    const WIDTH: f32 = 0.5;
    const HEIGHT: f32 = 0.5;
    const EYE_HEIGHT: f32 = 0.0;
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
