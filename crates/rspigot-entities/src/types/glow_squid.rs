use crate::{
    CreatureSound, CreatureSounds, EntityType, EntityTypeId, MobCategory, MobClass, MobSpawn,
    SoundSource, SoundVolume, SpawnHeightmap, SpawnPlacement, SpawnPlacementType, SpawnPredicate,
    VoicePitch,
};

pub struct GlowSquid;

impl EntityType for GlowSquid {
    const NAME: &'static str = "minecraft:glow_squid";
    const ID: EntityTypeId = EntityTypeId(62);
    const CLASS: &'static str = "GlowSquid";
    const CATEGORY: MobCategory = MobCategory::UndergroundWaterCreature;
    const WIDTH: f32 = 0.8;
    const HEIGHT: f32 = 0.8;
    const EYE_HEIGHT: f32 = 0.4;
    const FIRE_IMMUNE: bool = false;
    const SUMMONABLE: bool = true;
    const CAN_SERIALIZE: bool = true;
    const ALLOWED_IN_PEACEFUL: bool = true;
    const CAN_SPAWN_FAR_FROM_PLAYER: bool = false;
    const CLIENT_TRACKING_RANGE: u8 = 10;
    const UPDATE_INTERVAL: Option<u8> = Some(3);
    const TRACK_DELTAS: bool = true;
    const SPAWN_EGG: Option<u16> = Some(1305);
    const LIVING: bool = true;
    const ANCESTORS: &'static [&'static str] = &[
        "GlowSquid",
        "Squid",
        "AgeableWaterCreature",
        "AgeableMob",
        "PathfinderMob",
        "Mob",
        "LivingEntity",
        "Entity",
    ];
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
        ("minecraft:follow_range", 16.0),
        ("minecraft:friction_modifier", 1.0),
        ("minecraft:gravity", 0.08),
        ("minecraft:jump_strength", 0.41999998688697815),
        ("minecraft:knockback_resistance", 0.0),
        ("minecraft:max_absorption", 0.0),
        ("minecraft:max_health", 10.0),
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
    const SOUNDS: Option<CreatureSounds> = Some(CreatureSounds {
        hurt: CreatureSound::Fixed(Some("minecraft:entity.glow_squid.hurt")),
        death: CreatureSound::Fixed(Some("minecraft:entity.glow_squid.death")),
        small_fall: "minecraft:entity.generic.small_fall",
        big_fall: "minecraft:entity.generic.big_fall",
        volume: SoundVolume::Fixed(0.4),
        pitch: VoicePitch::Standard,
        source: SoundSource::Neutral,
        can_be_baby: true,
    });
    const MOB_SPAWN: Option<MobSpawn> = Some(MobSpawn {
        placement: Some(SpawnPlacement {
            kind: SpawnPlacementType::InWater,
            heightmap: SpawnHeightmap::MotionBlockingNoLeaves,
            predicate: SpawnPredicate::GlowSquid,
        }),
        check_spawn_rules: MobClass::PathfinderMob,
        walk_target_value: Some(MobClass::PathfinderMob),
        check_spawn_obstruction: MobClass::AgeableWaterCreature,
        max_spawn_cluster_size: Some(4),
        max_spawn_cluster_size_owner: MobClass::Mob,
        is_max_group_size_reached: MobClass::Mob,
        remove_when_far_away: MobClass::Mob,
        requires_custom_persistence: MobClass::Mob,
        spawn_dimensions_scale: 1.0,
        immune_to: "minecraft:default_immune_to",
    });
    type Behavior = crate::family::Plain;
}
