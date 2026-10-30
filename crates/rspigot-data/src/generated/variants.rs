use crate::tags::HolderSet;
use crate::variants::{MobVariant, SoundSet, SoundVariant, SpawnCondition};

pub(crate) static COW_VARIANTS: [MobVariant; 3] = [
    MobVariant {
        name: "minecraft:cold",
        spawn_conditions: &[
            SpawnCondition {
                biomes: Some(HolderSet::Tag("minecraft:spawns_cold_variant_farm_animals")),
                priority: 1,
            },
        ],
    },
    MobVariant {
        name: "minecraft:temperate",
        spawn_conditions: &[
            SpawnCondition {
                biomes: None,
                priority: 0,
            },
        ],
    },
    MobVariant {
        name: "minecraft:warm",
        spawn_conditions: &[
            SpawnCondition {
                biomes: Some(HolderSet::Tag("minecraft:spawns_warm_variant_farm_animals")),
                priority: 1,
            },
        ],
    },
];

pub(crate) static PIG_VARIANTS: [MobVariant; 3] = [
    MobVariant {
        name: "minecraft:cold",
        spawn_conditions: &[
            SpawnCondition {
                biomes: Some(HolderSet::Tag("minecraft:spawns_cold_variant_farm_animals")),
                priority: 1,
            },
        ],
    },
    MobVariant {
        name: "minecraft:temperate",
        spawn_conditions: &[
            SpawnCondition {
                biomes: None,
                priority: 0,
            },
        ],
    },
    MobVariant {
        name: "minecraft:warm",
        spawn_conditions: &[
            SpawnCondition {
                biomes: Some(HolderSet::Tag("minecraft:spawns_warm_variant_farm_animals")),
                priority: 1,
            },
        ],
    },
];

pub(crate) static CHICKEN_VARIANTS: [MobVariant; 3] = [
    MobVariant {
        name: "minecraft:cold",
        spawn_conditions: &[
            SpawnCondition {
                biomes: Some(HolderSet::Tag("minecraft:spawns_cold_variant_farm_animals")),
                priority: 1,
            },
        ],
    },
    MobVariant {
        name: "minecraft:temperate",
        spawn_conditions: &[
            SpawnCondition {
                biomes: None,
                priority: 0,
            },
        ],
    },
    MobVariant {
        name: "minecraft:warm",
        spawn_conditions: &[
            SpawnCondition {
                biomes: Some(HolderSet::Tag("minecraft:spawns_warm_variant_farm_animals")),
                priority: 1,
            },
        ],
    },
];

pub(crate) static COW_SOUND_VARIANTS: [SoundVariant; 2] = [
    SoundVariant {
        name: "minecraft:classic",
        adult: SoundSet {
            ambient: Some("minecraft:entity.cow.ambient"),
            death: Some("minecraft:entity.cow.death"),
            eat: None,
            hurt: Some("minecraft:entity.cow.hurt"),
            step: Some("minecraft:entity.cow.step"),
        },
        baby: SoundSet {
            ambient: Some("minecraft:entity.cow.ambient"),
            death: Some("minecraft:entity.cow.death"),
            eat: None,
            hurt: Some("minecraft:entity.cow.hurt"),
            step: Some("minecraft:entity.cow.step"),
        },
    },
    SoundVariant {
        name: "minecraft:moody",
        adult: SoundSet {
            ambient: Some("minecraft:entity.cow_moody.ambient"),
            death: Some("minecraft:entity.cow_moody.death"),
            eat: None,
            hurt: Some("minecraft:entity.cow_moody.hurt"),
            step: Some("minecraft:entity.cow_moody.step"),
        },
        baby: SoundSet {
            ambient: Some("minecraft:entity.cow_moody.ambient"),
            death: Some("minecraft:entity.cow_moody.death"),
            eat: None,
            hurt: Some("minecraft:entity.cow_moody.hurt"),
            step: Some("minecraft:entity.cow_moody.step"),
        },
    },
];

pub(crate) static PIG_SOUND_VARIANTS: [SoundVariant; 3] = [
    SoundVariant {
        name: "minecraft:big",
        adult: SoundSet {
            ambient: Some("minecraft:entity.pig_big.ambient"),
            death: Some("minecraft:entity.pig_big.death"),
            eat: Some("minecraft:entity.pig_big.eat"),
            hurt: Some("minecraft:entity.pig_big.hurt"),
            step: Some("minecraft:entity.pig.step"),
        },
        baby: SoundSet {
            ambient: Some("minecraft:entity.baby_pig.ambient"),
            death: Some("minecraft:entity.baby_pig.death"),
            eat: Some("minecraft:entity.baby_pig.eat"),
            hurt: Some("minecraft:entity.baby_pig.hurt"),
            step: Some("minecraft:entity.baby_pig.step"),
        },
    },
    SoundVariant {
        name: "minecraft:classic",
        adult: SoundSet {
            ambient: Some("minecraft:entity.pig.ambient"),
            death: Some("minecraft:entity.pig.death"),
            eat: Some("minecraft:entity.pig.eat"),
            hurt: Some("minecraft:entity.pig.hurt"),
            step: Some("minecraft:entity.pig.step"),
        },
        baby: SoundSet {
            ambient: Some("minecraft:entity.baby_pig.ambient"),
            death: Some("minecraft:entity.baby_pig.death"),
            eat: Some("minecraft:entity.baby_pig.eat"),
            hurt: Some("minecraft:entity.baby_pig.hurt"),
            step: Some("minecraft:entity.baby_pig.step"),
        },
    },
    SoundVariant {
        name: "minecraft:mini",
        adult: SoundSet {
            ambient: Some("minecraft:entity.pig_mini.ambient"),
            death: Some("minecraft:entity.pig_mini.death"),
            eat: Some("minecraft:entity.pig_mini.eat"),
            hurt: Some("minecraft:entity.pig_mini.hurt"),
            step: Some("minecraft:entity.pig.step"),
        },
        baby: SoundSet {
            ambient: Some("minecraft:entity.baby_pig.ambient"),
            death: Some("minecraft:entity.baby_pig.death"),
            eat: Some("minecraft:entity.baby_pig.eat"),
            hurt: Some("minecraft:entity.baby_pig.hurt"),
            step: Some("minecraft:entity.baby_pig.step"),
        },
    },
];

pub(crate) static CHICKEN_SOUND_VARIANTS: [SoundVariant; 2] = [
    SoundVariant {
        name: "minecraft:classic",
        adult: SoundSet {
            ambient: Some("minecraft:entity.chicken.ambient"),
            death: Some("minecraft:entity.chicken.death"),
            eat: None,
            hurt: Some("minecraft:entity.chicken.hurt"),
            step: Some("minecraft:entity.chicken.step"),
        },
        baby: SoundSet {
            ambient: Some("minecraft:entity.baby_chicken.ambient"),
            death: Some("minecraft:entity.baby_chicken.death"),
            eat: None,
            hurt: Some("minecraft:entity.baby_chicken.hurt"),
            step: Some("minecraft:entity.baby_chicken.step"),
        },
    },
    SoundVariant {
        name: "minecraft:picky",
        adult: SoundSet {
            ambient: Some("minecraft:entity.chicken_picky.ambient"),
            death: Some("minecraft:entity.chicken_picky.death"),
            eat: None,
            hurt: Some("minecraft:entity.chicken_picky.hurt"),
            step: Some("minecraft:entity.chicken.step"),
        },
        baby: SoundSet {
            ambient: Some("minecraft:entity.baby_chicken.ambient"),
            death: Some("minecraft:entity.baby_chicken.death"),
            eat: None,
            hurt: Some("minecraft:entity.baby_chicken.hurt"),
            step: Some("minecraft:entity.baby_chicken.step"),
        },
    },
];

pub(crate) static DYE_MIXES: [(u8, u8, u8); 18] = [
    (0, 7, 8), (0, 11, 3), (0, 13, 5), (0, 14, 6), (0, 15, 7), (4, 14, 1), (6, 10, 2), (7, 0, 8),
    (10, 6, 2), (11, 0, 3), (11, 13, 9), (11, 14, 10), (13, 0, 5), (13, 11, 9), (14, 0, 6),
    (14, 4, 1), (14, 11, 10), (15, 0, 7),
];
