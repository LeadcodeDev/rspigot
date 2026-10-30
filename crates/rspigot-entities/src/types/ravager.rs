use crate::{EntityType, EntityTypeId, MobCategory};

pub struct Ravager;

impl EntityType for Ravager {
    const NAME: &'static str = "minecraft:ravager";
    const ID: EntityTypeId = EntityTypeId(112);
    const CLASS: &'static str = "Ravager";
    const CATEGORY: MobCategory = MobCategory::Monster;
    const WIDTH: f32 = 1.95;
    const HEIGHT: f32 = 2.2;
    const EYE_HEIGHT: f32 = 1.8700001;
    const FIRE_IMMUNE: bool = false;
    const SUMMONABLE: bool = true;
    const CAN_SERIALIZE: bool = true;
    const ALLOWED_IN_PEACEFUL: bool = false;
    const CAN_SPAWN_FAR_FROM_PLAYER: bool = false;
    const CLIENT_TRACKING_RANGE: u8 = 10;
    const UPDATE_INTERVAL: Option<u8> = Some(3);
    const TRACK_DELTAS: bool = true;
    const SPAWN_EGG: Option<u16> = Some(1351);
    const LIVING: bool = true;
    const ANCESTORS: &'static [&'static str] = &[
        "Ravager",
        "Raider",
        "PatrollingMonster",
        "Monster",
        "PathfinderMob",
        "Mob",
        "LivingEntity",
        "Entity",
    ];
    const DEFAULT_ATTRIBUTES: &'static [(&'static str, f64)] = &[
        ("minecraft:air_drag_modifier", 1.0),
        ("minecraft:armor", 0.0),
        ("minecraft:armor_toughness", 0.0),
        ("minecraft:attack_damage", 12.0),
        ("minecraft:attack_knockback", 1.5),
        ("minecraft:below_name_distance", 10.0),
        ("minecraft:bounciness", 0.0),
        ("minecraft:burning_time", 1.0),
        ("minecraft:camera_distance", 4.0),
        ("minecraft:explosion_knockback_resistance", 0.0),
        ("minecraft:entity_interaction_range", 3.0),
        ("minecraft:fall_damage_multiplier", 1.0),
        ("minecraft:follow_range", 32.0),
        ("minecraft:friction_modifier", 1.0),
        ("minecraft:gravity", 0.08),
        ("minecraft:jump_strength", 0.41999998688697815),
        ("minecraft:knockback_resistance", 0.75),
        ("minecraft:max_absorption", 0.0),
        ("minecraft:max_health", 100.0),
        ("minecraft:movement_efficiency", 0.0),
        ("minecraft:movement_speed", 0.3),
        ("minecraft:name_tag_distance", 64.0),
        ("minecraft:oxygen_bonus", 0.0),
        ("minecraft:safe_fall_distance", 3.0),
        ("minecraft:scale", 1.0),
        ("minecraft:step_height", 1.0),
        ("minecraft:water_movement_efficiency", 0.0),
        ("minecraft:waypoint_transmit_range", 0.0),
    ];
    type Behavior = crate::family::Plain;
}
