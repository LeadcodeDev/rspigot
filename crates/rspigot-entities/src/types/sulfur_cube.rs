use crate::{
    CreatureSound, CreatureSounds, EntityType, EntityTypeId, MobCategory, MobClass, MobSpawn,
    SoundSource, SoundVolume, SpawnHeightmap, SpawnPlacement, SpawnPlacementType, SpawnPredicate,
    VoicePitch,
};

pub struct SulfurCube;

impl EntityType for SulfurCube {
    const NAME: &'static str = "minecraft:sulfur_cube";
    const ID: EntityTypeId = EntityTypeId(133);
    const CLASS: &'static str = "SulfurCube";
    const CATEGORY: MobCategory = MobCategory::Monster;
    const WIDTH: f32 = 0.49;
    const HEIGHT: f32 = 0.49;
    const EYE_HEIGHT: f32 = 0.175;
    const FIRE_IMMUNE: bool = false;
    const SUMMONABLE: bool = true;
    const CAN_SERIALIZE: bool = true;
    const ALLOWED_IN_PEACEFUL: bool = true;
    const CAN_SPAWN_FAR_FROM_PLAYER: bool = false;
    const CLIENT_TRACKING_RANGE: u8 = 10;
    const UPDATE_INTERVAL: Option<u8> = Some(3);
    const TRACK_DELTAS: bool = true;
    const SPAWN_EGG: Option<u16> = Some(1316);
    const LIVING: bool = true;
    const ANCESTORS: &'static [&'static str] = &[
        "SulfurCube",
        "AbstractCubeMob",
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
        ("minecraft:max_health", 20.0),
        ("minecraft:movement_efficiency", 0.0),
        ("minecraft:movement_speed", 0.7),
        ("minecraft:name_tag_distance", 64.0),
        ("minecraft:oxygen_bonus", 0.0),
        ("minecraft:safe_fall_distance", 3.0),
        ("minecraft:scale", 1.0),
        ("minecraft:step_height", 0.6),
        ("minecraft:tempt_range", 8.0),
        ("minecraft:water_movement_efficiency", 0.0),
        ("minecraft:waypoint_transmit_range", 0.0),
    ];
    const SOUNDS: Option<CreatureSounds> = Some(CreatureSounds {
        hurt: CreatureSound::Varied(&[
            Some("minecraft:entity.sulfur_cube.hurt"),
            Some("minecraft:entity.sulfur_cube.hurt"),
            Some("minecraft:entity.sulfur_cube.hurt"),
            Some("minecraft:entity.small_sulfur_cube.hurt"),
            Some("minecraft:entity.small_sulfur_cube.hurt"),
            Some("minecraft:entity.small_sulfur_cube.hurt"),
            Some("minecraft:entity.sulfur_cube.hurt"),
            Some("minecraft:entity.sulfur_cube.hurt"),
            Some("minecraft:entity.sulfur_cube.hurt"),
            Some("minecraft:entity.small_sulfur_cube.hurt"),
            Some("minecraft:entity.small_sulfur_cube.hurt"),
            Some("minecraft:entity.small_sulfur_cube.hurt"),
        ]),
        death: CreatureSound::Varied(&[
            Some("minecraft:entity.sulfur_cube.death"),
            Some("minecraft:entity.sulfur_cube.death"),
            Some("minecraft:entity.sulfur_cube.death"),
            Some("minecraft:entity.small_sulfur_cube.death"),
            Some("minecraft:entity.small_sulfur_cube.death"),
            Some("minecraft:entity.small_sulfur_cube.death"),
            Some("minecraft:entity.sulfur_cube.death"),
            Some("minecraft:entity.sulfur_cube.death"),
            Some("minecraft:entity.sulfur_cube.death"),
            Some("minecraft:entity.small_sulfur_cube.death"),
            Some("minecraft:entity.small_sulfur_cube.death"),
            Some("minecraft:entity.small_sulfur_cube.death"),
        ]),
        small_fall: "minecraft:entity.generic.small_fall",
        big_fall: "minecraft:entity.generic.big_fall",
        volume: SoundVolume::PerSize(0.4),
        pitch: VoicePitch::Standard,
        source: SoundSource::Neutral,
        can_be_baby: true,
    });
    const MOB_SPAWN: Option<MobSpawn> = Some(MobSpawn {
        placement: Some(SpawnPlacement {
            kind: SpawnPlacementType::OnGround,
            heightmap: SpawnHeightmap::MotionBlockingNoLeaves,
            predicate: SpawnPredicate::SulfurCube,
        }),
        check_spawn_rules: MobClass::PathfinderMob,
        walk_target_value: Some(MobClass::PathfinderMob),
        check_spawn_obstruction: MobClass::Mob,
        max_spawn_cluster_size: Some(4),
        max_spawn_cluster_size_owner: MobClass::Mob,
        is_max_group_size_reached: MobClass::Mob,
        remove_when_far_away: MobClass::Mob,
        requires_custom_persistence: MobClass::SulfurCube,
        spawn_dimensions_scale: 2.0,
        immune_to: "minecraft:default_immune_to",
    });
    type Behavior = crate::family::Plain;
}
