use crate::{EntityType, EntityTypeId, MobCategory};

pub struct LeashKnot;

impl EntityType for LeashKnot {
    const NAME: &'static str = "minecraft:leash_knot";
    const ID: EntityTypeId = EntityTypeId(77);
    const CLASS: &'static str = "LeashFenceKnotEntity";
    const CATEGORY: MobCategory = MobCategory::Misc;
    const WIDTH: f32 = 0.375;
    const HEIGHT: f32 = 0.5;
    const EYE_HEIGHT: f32 = 0.0625;
    const FIRE_IMMUNE: bool = false;
    const SUMMONABLE: bool = true;
    const CAN_SERIALIZE: bool = false;
    const ALLOWED_IN_PEACEFUL: bool = true;
    const CAN_SPAWN_FAR_FROM_PLAYER: bool = true;
    const CLIENT_TRACKING_RANGE: u8 = 10;
    const UPDATE_INTERVAL: Option<u8> = None;
    const TRACK_DELTAS: bool = false;
    const SPAWN_EGG: Option<u16> = None;
    const LIVING: bool = false;
    const ANCESTORS: &'static [&'static str] = &[
        "LeashFenceKnotEntity",
        "BlockAttachedEntity",
        "Entity",
    ];
    const DEFAULT_ATTRIBUTES: &'static [(&'static str, f64)] = &[];
    type Behavior = crate::family::Plain;
}
