use crate::{CreatureSounds, EntityType, EntityTypeId, MobCategory};

pub struct WindCharge;

impl EntityType for WindCharge {
    const NAME: &'static str = "minecraft:wind_charge";
    const ID: EntityTypeId = EntityTypeId(147);
    const CLASS: &'static str = "WindCharge";
    const CATEGORY: MobCategory = MobCategory::Misc;
    const WIDTH: f32 = 0.3125;
    const HEIGHT: f32 = 0.3125;
    const EYE_HEIGHT: f32 = 0.0;
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
        "WindCharge",
        "AbstractWindCharge",
        "AbstractHurtingProjectile",
        "Projectile",
        "Entity",
    ];
    const DEFAULT_ATTRIBUTES: &'static [(&'static str, f64)] = &[];
    const SOUNDS: Option<CreatureSounds> = None;
    type Behavior = crate::family::Plain;
}
