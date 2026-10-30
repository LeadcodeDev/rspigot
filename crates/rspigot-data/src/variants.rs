use crate::generated::variants::{
    CHICKEN_SOUND_VARIANTS, CHICKEN_VARIANTS, COW_SOUND_VARIANTS, COW_VARIANTS, DYE_MIXES,
    PIG_SOUND_VARIANTS, PIG_VARIANTS,
};
use crate::tags::HolderSet;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SpawnCondition {
    pub biomes: Option<HolderSet>,
    pub priority: i32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MobVariant {
    pub name: &'static str,
    pub spawn_conditions: &'static [SpawnCondition],
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SoundSet {
    pub ambient: Option<&'static str>,
    pub death: Option<&'static str>,
    pub eat: Option<&'static str>,
    pub hurt: Option<&'static str>,
    pub step: Option<&'static str>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SoundVariant {
    pub name: &'static str,
    pub adult: SoundSet,
    pub baby: SoundSet,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Farm {
    Cow,
    Pig,
    Chicken,
}

impl Farm {
    pub fn variants(self) -> &'static [MobVariant] {
        match self {
            Self::Cow => &COW_VARIANTS,
            Self::Pig => &PIG_VARIANTS,
            Self::Chicken => &CHICKEN_VARIANTS,
        }
    }

    pub fn sound_variants(self) -> &'static [SoundVariant] {
        match self {
            Self::Cow => &COW_SOUND_VARIANTS,
            Self::Pig => &PIG_SOUND_VARIANTS,
            Self::Chicken => &CHICKEN_SOUND_VARIANTS,
        }
    }

    pub fn variant_index(self, name: &str) -> Option<u32> {
        index_of(self.variants().iter().map(|v| v.name), name)
    }

    pub fn sound_variant_index(self, name: &str) -> Option<u32> {
        index_of(self.sound_variants().iter().map(|v| v.name), name)
    }
}

fn index_of<'a>(names: impl Iterator<Item = &'a str>, name: &str) -> Option<u32> {
    names
        .enumerate()
        .find(|(_, candidate)| *candidate == name)
        .and_then(|(index, _)| u32::try_from(index).ok())
}

pub fn dye_mix(first: u8, second: u8) -> Option<u8> {
    DYE_MIXES
        .binary_search_by(|(a, b, _)| (*a, *b).cmp(&(first, second)))
        .ok()
        .map(|index| DYE_MIXES[index].2)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::registries;

    #[test]
    fn variants_follow_the_synced_registry_order() {
        for (farm, registry, sounds) in [
            (
                Farm::Cow,
                "minecraft:cow_variant",
                "minecraft:cow_sound_variant",
            ),
            (
                Farm::Pig,
                "minecraft:pig_variant",
                "minecraft:pig_sound_variant",
            ),
            (
                Farm::Chicken,
                "minecraft:chicken_variant",
                "minecraft:chicken_sound_variant",
            ),
        ] {
            let names: Vec<&str> = farm.variants().iter().map(|v| v.name).collect();
            assert_eq!(Some(names.as_slice()), registries::synced_entries(registry));
            let names: Vec<&str> = farm.sound_variants().iter().map(|v| v.name).collect();
            assert_eq!(Some(names.as_slice()), registries::synced_entries(sounds));
        }
        assert_eq!(Farm::Cow.variant_index("minecraft:temperate"), Some(1));
        assert_eq!(Farm::Pig.sound_variant_index("minecraft:classic"), Some(1));
    }

    #[test]
    fn farm_variants_spawn_by_biome_tag() {
        let cold = Farm::Cow.variants()[0];
        assert_eq!(cold.name, "minecraft:cold");
        assert_eq!(
            cold.spawn_conditions,
            &[SpawnCondition {
                biomes: Some(HolderSet::Tag("minecraft:spawns_cold_variant_farm_animals")),
                priority: 1,
            }]
        );
        let temperate = Farm::Chicken.variants()[1];
        assert_eq!(
            temperate.spawn_conditions,
            &[SpawnCondition {
                biomes: None,
                priority: 0
            }]
        );
    }

    #[test]
    fn sound_sets_split_adults_and_babies() {
        let big = Farm::Pig.sound_variants()[0];
        assert_eq!(big.adult.ambient, Some("minecraft:entity.pig_big.ambient"));
        assert_eq!(big.baby.ambient, Some("minecraft:entity.baby_pig.ambient"));
        let moody = Farm::Cow.sound_variants()[1];
        assert_eq!(moody.adult.hurt, Some("minecraft:entity.cow_moody.hurt"));
        assert_eq!(moody.baby, moody.adult);
    }

    #[test]
    fn dye_mixes_come_from_two_dye_recipes() {
        assert_eq!(dye_mix(14, 4), Some(1));
        assert_eq!(dye_mix(4, 14), Some(1));
        assert_eq!(dye_mix(15, 0), Some(7));
        assert_eq!(dye_mix(0, 0), None);
        assert_eq!(dye_mix(11, 13), Some(9));
    }
}
