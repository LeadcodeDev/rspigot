use crate::{CreatureSounds, EntityType, EntityTypeId, MobCategory};

pub struct ItemFrame;

impl EntityType for ItemFrame {
    const NAME: &'static str = "minecraft:item_frame";
    const ID: EntityTypeId = EntityTypeId(74);
    const CLASS: &'static str = "ItemFrame";
    const CATEGORY: MobCategory = MobCategory::Misc;
    const WIDTH: f32 = 0.5;
    const HEIGHT: f32 = 0.5;
    const EYE_HEIGHT: f32 = 0.0;
    const FIRE_IMMUNE: bool = false;
    const SUMMONABLE: bool = true;
    const CAN_SERIALIZE: bool = true;
    const ALLOWED_IN_PEACEFUL: bool = true;
    const CAN_SPAWN_FAR_FROM_PLAYER: bool = true;
    const CLIENT_TRACKING_RANGE: u8 = 10;
    const UPDATE_INTERVAL: Option<u8> = None;
    const TRACK_DELTAS: bool = false;
    const SPAWN_EGG: Option<u16> = None;
    const LIVING: bool = false;
    const ANCESTORS: &'static [&'static str] = &[
        "ItemFrame",
        "HangingEntity",
        "BlockAttachedEntity",
        "Entity",
    ];
    const DEFAULT_ATTRIBUTES: &'static [(&'static str, f64)] = &[];
    const SOUNDS: Option<CreatureSounds> = None;
    type Behavior = crate::family::Plain;
}
