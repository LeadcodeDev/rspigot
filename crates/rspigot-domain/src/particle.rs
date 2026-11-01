use crate::{BlockStateId, ItemId, Vec3};

pub const BLOCK_PARTICLES: [&str; 5] = [
    "minecraft:block",
    "minecraft:block_marker",
    "minecraft:falling_dust",
    "minecraft:dust_pillar",
    "minecraft:block_crumble",
];

pub const PARTICLES_WITH_DATA: [&str; 22] = [
    "minecraft:block",
    "minecraft:block_marker",
    "minecraft:geyser",
    "minecraft:geyser_base",
    "minecraft:geyser_poof",
    "minecraft:geyser_plume",
    "minecraft:dragon_breath",
    "minecraft:dust",
    "minecraft:dust_color_transition",
    "minecraft:effect",
    "minecraft:entity_effect",
    "minecraft:falling_dust",
    "minecraft:tinted_leaves",
    "minecraft:sculk_charge",
    "minecraft:flash",
    "minecraft:instant_effect",
    "minecraft:item",
    "minecraft:vibration",
    "minecraft:trail",
    "minecraft:shriek",
    "minecraft:dust_pillar",
    "minecraft:block_crumble",
];

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ParticleData {
    None,
    Item(ItemId),
    Block(BlockStateId),
    Dust { color: u32, scale: f32 },
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Particle {
    pub kind: &'static str,
    pub data: ParticleData,
    pub position: Vec3,
    pub count: i32,
    pub spread: Vec3,
    pub speed: f64,
}

impl Particle {
    pub const fn single(kind: &'static str, position: Vec3) -> Self {
        Self {
            kind,
            data: ParticleData::None,
            position,
            count: 1,
            spread: Vec3::ZERO,
            speed: 0.0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_data_particle_is_a_registered_particle_type() {
        for name in PARTICLES_WITH_DATA.iter().chain(&BLOCK_PARTICLES) {
            assert!(
                rspigot_data::registries::static_index("minecraft:particle_type", name).is_some(),
                "{name}"
            );
        }
        assert!(
            BLOCK_PARTICLES
                .iter()
                .all(|name| PARTICLES_WITH_DATA.contains(name))
        );
    }
}
