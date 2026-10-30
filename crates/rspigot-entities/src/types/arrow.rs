use crate::{CreatureSounds, EntityType, EntityTypeId, MobCategory, MobSpawn};

pub struct Arrow;

impl EntityType for Arrow {
    const NAME: &'static str = "minecraft:arrow";
    const ID: EntityTypeId = EntityTypeId(6);
    const CLASS: &'static str = "Arrow";
    const CATEGORY: MobCategory = MobCategory::Misc;
    const WIDTH: f32 = 0.5;
    const HEIGHT: f32 = 0.5;
    const EYE_HEIGHT: f32 = 0.13;
    const FIRE_IMMUNE: bool = false;
    const SUMMONABLE: bool = true;
    const CAN_SERIALIZE: bool = true;
    const ALLOWED_IN_PEACEFUL: bool = true;
    const CAN_SPAWN_FAR_FROM_PLAYER: bool = true;
    const CLIENT_TRACKING_RANGE: u8 = 4;
    const UPDATE_INTERVAL: Option<u8> = Some(20);
    const TRACK_DELTAS: bool = true;
    const SPAWN_EGG: Option<u16> = None;
    const LIVING: bool = false;
    const ANCESTORS: &'static [&'static str] = &["Arrow", "AbstractArrow", "Projectile", "Entity"];
    const DEFAULT_ATTRIBUTES: &'static [(&'static str, f64)] = &[];
    const SOUNDS: Option<CreatureSounds> = None;
    const MOB_SPAWN: Option<MobSpawn> = None;
    type Behavior = crate::family::Plain;
}
