use crate::{EntityType, EntityTypeId, MobCategory};

pub struct Mannequin;

impl EntityType for Mannequin {
    const NAME: &'static str = "minecraft:mannequin";
    const ID: EntityTypeId = EntityTypeId(84);
    const CLASS: &'static str = "Mannequin";
    const CATEGORY: MobCategory = MobCategory::Misc;
    const WIDTH: f32 = 0.6;
    const HEIGHT: f32 = 1.8;
    const EYE_HEIGHT: f32 = 1.62;
    const FIRE_IMMUNE: bool = false;
    const SUMMONABLE: bool = true;
    const CAN_SPAWN_FAR_FROM_PLAYER: bool = true;
    const CLIENT_TRACKING_RANGE: u8 = 32;
    const UPDATE_INTERVAL: Option<u8> = Some(2);
    const SPAWN_EGG: Option<u16> = None;
    const LIVING: bool = true;
    const DEFAULT_ATTRIBUTES: &'static [(&'static str, f64)] = &[
        ("minecraft:air_drag_modifier", 1.0),
        ("minecraft:armor", 0.0),
        ("minecraft:armor_toughness", 0.0),
        ("minecraft:attack_knockback", 0.0),
        ("minecraft:below_name_distance", 10.0),
        ("minecraft:bounciness", 0.0),
        ("minecraft:burning_time", 1.0),
        ("minecraft:camera_distance", 4.0),
        ("minecraft:explosion_knockback_resistance", 0.0),
        ("minecraft:entity_interaction_range", 3.0),
        ("minecraft:fall_damage_multiplier", 1.0),
        ("minecraft:friction_modifier", 1.0),
        ("minecraft:gravity", 0.08),
        ("minecraft:jump_strength", 0.41999998688697815),
        ("minecraft:knockback_resistance", 0.0),
        ("minecraft:max_absorption", 0.0),
        ("minecraft:max_health", 20.0),
        ("minecraft:movement_efficiency", 0.0),
        ("minecraft:movement_speed", 0.7),
        ("minecraft:name_tag_distance", 64.0),
        ("minecraft:oxygen_bonus", 0.0),
        ("minecraft:safe_fall_distance", 3.0),
        ("minecraft:scale", 1.0),
        ("minecraft:step_height", 0.6),
        ("minecraft:water_movement_efficiency", 0.0),
        ("minecraft:waypoint_transmit_range", 0.0),
    ];
    type Behavior = crate::family::Plain;
}
