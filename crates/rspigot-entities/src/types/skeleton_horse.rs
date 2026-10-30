use crate::{EntityType, EntityTypeId, MobCategory};

pub struct SkeletonHorse;

impl EntityType for SkeletonHorse {
    const NAME: &'static str = "minecraft:skeleton_horse";
    const ID: EntityTypeId = EntityTypeId(119);
    const CLASS: &'static str = "SkeletonHorse";
    const CATEGORY: MobCategory = MobCategory::Creature;
    const WIDTH: f32 = 1.3964844;
    const HEIGHT: f32 = 1.6;
    const EYE_HEIGHT: f32 = 1.52;
    const FIRE_IMMUNE: bool = false;
    const SUMMONABLE: bool = true;
    const CAN_SPAWN_FAR_FROM_PLAYER: bool = true;
    const CLIENT_TRACKING_RANGE: u8 = 10;
    const UPDATE_INTERVAL: Option<u8> = Some(3);
    const SPAWN_EGG: Option<u16> = Some(1329);
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
        ("minecraft:fall_damage_multiplier", 0.5),
        ("minecraft:follow_range", 16.0),
        ("minecraft:friction_modifier", 1.0),
        ("minecraft:gravity", 0.08),
        ("minecraft:jump_strength", 0.7),
        ("minecraft:knockback_resistance", 0.0),
        ("minecraft:max_absorption", 0.0),
        ("minecraft:max_health", 15.0),
        ("minecraft:movement_efficiency", 0.0),
        ("minecraft:movement_speed", 0.20000000298023224),
        ("minecraft:name_tag_distance", 64.0),
        ("minecraft:oxygen_bonus", 0.0),
        ("minecraft:safe_fall_distance", 6.0),
        ("minecraft:scale", 1.0),
        ("minecraft:step_height", 1.0),
        ("minecraft:tempt_range", 10.0),
        ("minecraft:water_movement_efficiency", 0.0),
        ("minecraft:waypoint_transmit_range", 0.0),
    ];
    type Behavior = crate::family::Plain;
}
