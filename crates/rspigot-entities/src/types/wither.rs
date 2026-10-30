use crate::{
    CreatureSound, CreatureSounds, EntityType, EntityTypeId, MobCategory, SoundSource, SoundVolume,
    VoicePitch,
};

pub struct Wither;

impl EntityType for Wither {
    const NAME: &'static str = "minecraft:wither";
    const ID: EntityTypeId = EntityTypeId(149);
    const CLASS: &'static str = "WitherBoss";
    const CATEGORY: MobCategory = MobCategory::Monster;
    const WIDTH: f32 = 0.9;
    const HEIGHT: f32 = 3.5;
    const EYE_HEIGHT: f32 = 2.9750001;
    const FIRE_IMMUNE: bool = true;
    const SUMMONABLE: bool = true;
    const CAN_SERIALIZE: bool = true;
    const ALLOWED_IN_PEACEFUL: bool = false;
    const CAN_SPAWN_FAR_FROM_PLAYER: bool = false;
    const CLIENT_TRACKING_RANGE: u8 = 10;
    const UPDATE_INTERVAL: Option<u8> = Some(3);
    const TRACK_DELTAS: bool = false;
    const SPAWN_EGG: Option<u16> = Some(1331);
    const LIVING: bool = true;
    const ANCESTORS: &'static [&'static str] = &[
        "WitherBoss",
        "Monster",
        "PathfinderMob",
        "Mob",
        "LivingEntity",
        "Entity",
    ];
    const DEFAULT_ATTRIBUTES: &'static [(&'static str, f64)] = &[
        ("minecraft:air_drag_modifier", 1.0),
        ("minecraft:armor", 4.0),
        ("minecraft:armor_toughness", 0.0),
        ("minecraft:attack_damage", 2.0),
        ("minecraft:attack_knockback", 0.0),
        ("minecraft:below_name_distance", 10.0),
        ("minecraft:bounciness", 0.0),
        ("minecraft:burning_time", 1.0),
        ("minecraft:camera_distance", 4.0),
        ("minecraft:explosion_knockback_resistance", 0.0),
        ("minecraft:entity_interaction_range", 3.0),
        ("minecraft:fall_damage_multiplier", 1.0),
        ("minecraft:flying_speed", 0.6000000238418579),
        ("minecraft:follow_range", 40.0),
        ("minecraft:friction_modifier", 1.0),
        ("minecraft:gravity", 0.08),
        ("minecraft:jump_strength", 0.41999998688697815),
        ("minecraft:knockback_resistance", 0.0),
        ("minecraft:max_absorption", 0.0),
        ("minecraft:max_health", 300.0),
        ("minecraft:movement_efficiency", 0.0),
        ("minecraft:movement_speed", 0.6000000238418579),
        ("minecraft:name_tag_distance", 64.0),
        ("minecraft:oxygen_bonus", 0.0),
        ("minecraft:safe_fall_distance", 3.0),
        ("minecraft:scale", 1.0),
        ("minecraft:step_height", 0.6),
        ("minecraft:water_movement_efficiency", 0.0),
        ("minecraft:waypoint_transmit_range", 0.0),
    ];
    const SOUNDS: Option<CreatureSounds> = Some(CreatureSounds {
        hurt: CreatureSound::Fixed(Some("minecraft:entity.wither.hurt")),
        death: CreatureSound::Fixed(Some("minecraft:entity.wither.death")),
        small_fall: "minecraft:entity.hostile.small_fall",
        big_fall: "minecraft:entity.hostile.big_fall",
        volume: SoundVolume::Fixed(1.0),
        pitch: VoicePitch::Standard,
        source: SoundSource::Hostile,
        can_be_baby: false,
    });
    type Behavior = crate::family::Plain;
}
