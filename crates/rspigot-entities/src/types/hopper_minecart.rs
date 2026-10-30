use crate::{CreatureSounds, EntityType, EntityTypeId, MobCategory};

pub struct HopperMinecart;

impl EntityType for HopperMinecart {
    const NAME: &'static str = "minecraft:hopper_minecart";
    const ID: EntityTypeId = EntityTypeId(66);
    const CLASS: &'static str = "MinecartHopper";
    const CATEGORY: MobCategory = MobCategory::Misc;
    const WIDTH: f32 = 0.98;
    const HEIGHT: f32 = 0.7;
    const EYE_HEIGHT: f32 = 0.595;
    const FIRE_IMMUNE: bool = false;
    const SUMMONABLE: bool = true;
    const CAN_SERIALIZE: bool = true;
    const ALLOWED_IN_PEACEFUL: bool = true;
    const CAN_SPAWN_FAR_FROM_PLAYER: bool = true;
    const CLIENT_TRACKING_RANGE: u8 = 8;
    const UPDATE_INTERVAL: Option<u8> = Some(3);
    const TRACK_DELTAS: bool = true;
    const SPAWN_EGG: Option<u16> = None;
    const LIVING: bool = false;
    const ANCESTORS: &'static [&'static str] = &[
        "MinecartHopper",
        "AbstractMinecartContainer",
        "AbstractMinecart",
        "VehicleEntity",
        "Entity",
    ];
    const DEFAULT_ATTRIBUTES: &'static [(&'static str, f64)] = &[];
    const SOUNDS: Option<CreatureSounds> = None;
    type Behavior = crate::family::Plain;
}
