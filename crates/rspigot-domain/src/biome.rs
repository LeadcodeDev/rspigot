use crate::BiomeId;

pub fn in_tag(biome: BiomeId, tag: &str) -> bool {
    rspigot_data::tags::tag("minecraft:worldgen/biome", tag)
        .is_some_and(|ids| ids.binary_search(&u32::from(biome.0)).is_ok())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn farm_animal_tags_resolve() {
        let snowy = BiomeId(rspigot_data::registries::biome_id("minecraft:snowy_plains").unwrap());
        let desert = BiomeId(rspigot_data::registries::biome_id("minecraft:desert").unwrap());
        assert!(in_tag(snowy, "minecraft:spawns_cold_variant_farm_animals"));
        assert!(in_tag(desert, "minecraft:spawns_warm_variant_farm_animals"));
        assert!(!in_tag(
            desert,
            "minecraft:spawns_cold_variant_farm_animals"
        ));
    }
}
