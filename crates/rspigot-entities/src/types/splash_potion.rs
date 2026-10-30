use crate::{EntityType, EntityTypeId, MobCategory};

pub struct SplashPotion;

impl EntityType for SplashPotion {
    const NAME: &'static str = "minecraft:splash_potion";
    const ID: EntityTypeId = EntityTypeId(108);
    const CLASS: &'static str = "ThrownSplashPotion";
    const CATEGORY: MobCategory = MobCategory::Misc;
    const WIDTH: f32 = 0.25;
    const HEIGHT: f32 = 0.25;
    const EYE_HEIGHT: f32 = 0.2125;
    const FIRE_IMMUNE: bool = false;
    const SUMMONABLE: bool = true;
    const CAN_SERIALIZE: bool = true;
    const ALLOWED_IN_PEACEFUL: bool = true;
    const CAN_SPAWN_FAR_FROM_PLAYER: bool = true;
    const CLIENT_TRACKING_RANGE: u8 = 4;
    const UPDATE_INTERVAL: Option<u8> = Some(10);
    const TRACK_DELTAS: bool = true;
    const SPAWN_EGG: Option<u16> = None;
    const LIVING: bool = false;
    const ANCESTORS: &'static [&'static str] = &[
        "ThrownSplashPotion",
        "AbstractThrownPotion",
        "ThrowableItemProjectile",
        "ThrowableProjectile",
        "Projectile",
        "Entity",
    ];
    const DEFAULT_ATTRIBUTES: &'static [(&'static str, f64)] = &[];
    type Behavior = crate::family::Plain;
}
