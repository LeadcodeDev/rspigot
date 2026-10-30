use crate::{EntityType, EntityTypeId, MobCategory};

pub struct ItemDisplay;

impl EntityType for ItemDisplay {
    const NAME: &'static str = "minecraft:item_display";
    const ID: EntityTypeId = EntityTypeId(73);
    const CLASS: &'static str = "ItemDisplay";
    const CATEGORY: MobCategory = MobCategory::Misc;
    const WIDTH: f32 = 0.0;
    const HEIGHT: f32 = 0.0;
    const EYE_HEIGHT: f32 = 0.0;
    const FIRE_IMMUNE: bool = false;
    const SUMMONABLE: bool = true;
    const CAN_SPAWN_FAR_FROM_PLAYER: bool = true;
    const CLIENT_TRACKING_RANGE: u8 = 10;
    const UPDATE_INTERVAL: Option<u8> = Some(1);
    const SPAWN_EGG: Option<u16> = None;
    const LIVING: bool = false;
    const DEFAULT_ATTRIBUTES: &'static [(&'static str, f64)] = &[];
    type Behavior = crate::family::Plain;
}
