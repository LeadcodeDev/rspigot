use crate::{
    CreatureSound, CreatureSounds, EntityType, EntityTypeId, MobCategory, MobClass, MobSpawn,
    SoundSource, SoundVolume, SpawnHeightmap, SpawnPlacement, SpawnPlacementType, SpawnPredicate,
    VoicePitch,
};

pub struct ZombifiedPiglin;

impl EntityType for ZombifiedPiglin {
    const NAME: &'static str = "minecraft:zombified_piglin";
    const ID: EntityTypeId = EntityTypeId(158);
    const CLASS: &'static str = "ZombifiedPiglin";
    const CATEGORY: MobCategory = MobCategory::Monster;
    const WIDTH: f32 = 0.6;
    const HEIGHT: f32 = 1.95;
    const EYE_HEIGHT: f32 = 1.79;
    const FIRE_IMMUNE: bool = true;
    const SUMMONABLE: bool = true;
    const CAN_SERIALIZE: bool = true;
    const ALLOWED_IN_PEACEFUL: bool = false;
    const CAN_SPAWN_FAR_FROM_PLAYER: bool = false;
    const CLIENT_TRACKING_RANGE: u8 = 8;
    const UPDATE_INTERVAL: Option<u8> = Some(3);
    const TRACK_DELTAS: bool = true;
    const SPAWN_EGG: Option<u16> = Some(1363);
    const LIVING: bool = true;
    const ANCESTORS: &'static [&'static str] = &[
        "ZombifiedPiglin",
        "Zombie",
        "Monster",
        "PathfinderMob",
        "Mob",
        "LivingEntity",
        "Entity",
    ];
    const DEFAULT_ATTRIBUTES: &'static [(&'static str, f64)] = &[
        ("minecraft:air_drag_modifier", 1.0),
        ("minecraft:armor", 2.0),
        ("minecraft:armor_toughness", 0.0),
        ("minecraft:attack_damage", 5.0),
        ("minecraft:attack_knockback", 0.0),
        ("minecraft:below_name_distance", 10.0),
        ("minecraft:bounciness", 0.0),
        ("minecraft:burning_time", 1.0),
        ("minecraft:camera_distance", 4.0),
        ("minecraft:explosion_knockback_resistance", 0.0),
        ("minecraft:entity_interaction_range", 3.0),
        ("minecraft:fall_damage_multiplier", 1.0),
        ("minecraft:follow_range", 35.0),
        ("minecraft:friction_modifier", 1.0),
        ("minecraft:gravity", 0.08),
        ("minecraft:jump_strength", 0.41999998688697815),
        ("minecraft:knockback_resistance", 0.0),
        ("minecraft:max_absorption", 0.0),
        ("minecraft:max_health", 20.0),
        ("minecraft:movement_efficiency", 0.0),
        ("minecraft:movement_speed", 0.23000000417232513),
        ("minecraft:name_tag_distance", 64.0),
        ("minecraft:oxygen_bonus", 0.0),
        ("minecraft:safe_fall_distance", 3.0),
        ("minecraft:scale", 1.0),
        ("minecraft:spawn_reinforcements", 0.0),
        ("minecraft:step_height", 0.6),
        ("minecraft:water_movement_efficiency", 0.0),
        ("minecraft:waypoint_transmit_range", 0.0),
    ];
    const SOUNDS: Option<CreatureSounds> = Some(CreatureSounds {
        hurt: CreatureSound::Fixed(Some("minecraft:entity.zombified_piglin.hurt")),
        death: CreatureSound::Fixed(Some("minecraft:entity.zombified_piglin.death")),
        small_fall: "minecraft:entity.hostile.small_fall",
        big_fall: "minecraft:entity.hostile.big_fall",
        volume: SoundVolume::Fixed(1.0),
        pitch: VoicePitch::Standard,
        source: SoundSource::Hostile,
        can_be_baby: true,
    });
    const MOB_SPAWN: Option<MobSpawn> = Some(MobSpawn {
        placement: Some(SpawnPlacement {
            kind: SpawnPlacementType::OnGround,
            heightmap: SpawnHeightmap::MotionBlockingNoLeaves,
            predicate: SpawnPredicate::ZombifiedPiglin,
        }),
        check_spawn_rules: MobClass::PathfinderMob,
        walk_target_value: Some(MobClass::Monster),
        check_spawn_obstruction: MobClass::ZombifiedPiglin,
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
