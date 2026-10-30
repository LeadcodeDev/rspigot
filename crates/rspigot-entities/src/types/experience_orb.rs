use crate::{EntityType, EntityTypeId, MobCategory};

pub struct ExperienceOrb;

impl EntityType for ExperienceOrb {
    const NAME: &'static str = "minecraft:experience_orb";
    const ID: EntityTypeId = EntityTypeId(50);
    const CLASS: &'static str = "ExperienceOrb";
    const CATEGORY: MobCategory = MobCategory::Misc;
    const WIDTH: f32 = 0.5;
    const HEIGHT: f32 = 0.5;
    const EYE_HEIGHT: f32 = 0.425;
    const FIRE_IMMUNE: bool = false;
    const SUMMONABLE: bool = true;
    const CAN_SERIALIZE: bool = true;
    const ALLOWED_IN_PEACEFUL: bool = true;
    const CAN_SPAWN_FAR_FROM_PLAYER: bool = true;
    const CLIENT_TRACKING_RANGE: u8 = 6;
    const UPDATE_INTERVAL: Option<u8> = Some(20);
    const TRACK_DELTAS: bool = true;
    const SPAWN_EGG: Option<u16> = None;
    const LIVING: bool = false;
    const ANCESTORS: &'static [&'static str] = &["ExperienceOrb", "Entity"];
    const DEFAULT_ATTRIBUTES: &'static [(&'static str, f64)] = &[];
    type Behavior = crate::family::Plain;
}
