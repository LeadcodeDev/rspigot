use crate::generated::climate::CLIMATE;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Climate {
    pub has_precipitation: bool,
    pub temperature: f32,
    pub frozen: bool,
}

pub fn climate(biome: u16) -> Option<Climate> {
    CLIMATE.get(usize::from(biome)).copied()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::registries::biome_id;

    #[test]
    fn biomes_carry_their_precipitation_and_temperature() {
        let plains = climate(biome_id("minecraft:plains").unwrap()).unwrap();
        assert!(plains.has_precipitation);
        assert_eq!(plains.temperature, 0.8);
        let desert = climate(biome_id("minecraft:desert").unwrap()).unwrap();
        assert!(!desert.has_precipitation);
        assert!(
            climate(biome_id("minecraft:frozen_ocean").unwrap())
                .unwrap()
                .frozen
        );
    }
}
