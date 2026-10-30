use rspigot_domain::BlockStateId;

use crate::resources::{Biome, BlockKind, Palette};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Ground {
    Grass,
    SnowyGrass,
    Sand,
    Bare,
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct Layers {
    pub(crate) top: BlockStateId,
    pub(crate) filler: BlockStateId,
    pub(crate) base: Option<BlockStateId>,
    pub(crate) filler_depth: i32,
    pub(crate) base_depth: i32,
    pub(crate) ground: Ground,
    pub(crate) snow_cover: bool,
}

fn plain(top: BlockStateId, filler: BlockStateId, filler_depth: i32, ground: Ground) -> Layers {
    Layers {
        top,
        filler,
        base: None,
        filler_depth,
        base_depth: 0,
        ground,
        snow_cover: false,
    }
}

fn seabed(palette: &Palette, biome: Biome, patch: f64) -> BlockStateId {
    let choices: &[BlockKind] = match biome {
        Biome::WarmOcean
        | Biome::LukewarmOcean
        | Biome::DeepLukewarmOcean
        | Biome::Desert
        | Biome::Beach => &[
            BlockKind::Sand,
            BlockKind::Sand,
            BlockKind::Sand,
            BlockKind::Clay,
        ],
        Biome::FrozenOcean
        | Biome::DeepFrozenOcean
        | Biome::ColdOcean
        | Biome::DeepColdOcean
        | Biome::FrozenRiver => &[
            BlockKind::Gravel,
            BlockKind::Gravel,
            BlockKind::Dirt,
            BlockKind::Sand,
        ],
        _ => &[
            BlockKind::Sand,
            BlockKind::Sand,
            BlockKind::Gravel,
            BlockKind::Dirt,
            BlockKind::Clay,
        ],
    };
    let unit = (patch * 1.4).clamp(-1.0, 1.0) * 0.5 + 0.5;
    let index = ((unit * choices.len() as f64) as usize).min(choices.len() - 1);
    palette[choices[index]]
}

pub(crate) fn layers(
    palette: &Palette,
    biome: Biome,
    height: i32,
    sea_level: i32,
    patch: f64,
    roll: u64,
) -> Layers {
    let dirt_depth = 3 + (roll % 3) as i32;
    if height < sea_level {
        let floor = seabed(palette, biome, patch);
        return plain(floor, floor, 3, Ground::Bare);
    }
    let grass = |snowy: bool| {
        let top = if snowy {
            palette[BlockKind::SnowyGrassBlock]
        } else {
            palette[BlockKind::GrassBlock]
        };
        let ground = if snowy {
            Ground::SnowyGrass
        } else {
            Ground::Grass
        };
        let mut layers = plain(top, palette[BlockKind::Dirt], dirt_depth, ground);
        layers.snow_cover = snowy;
        layers
    };
    let sand = palette[BlockKind::Sand];
    match biome {
        Biome::Desert => Layers {
            base: Some(palette[BlockKind::Sandstone]),
            base_depth: 4,
            ..plain(sand, sand, dirt_depth, Ground::Sand)
        },
        Biome::Beach
        | Biome::Ocean
        | Biome::DeepOcean
        | Biome::ColdOcean
        | Biome::DeepColdOcean
        | Biome::FrozenOcean
        | Biome::DeepFrozenOcean
        | Biome::LukewarmOcean
        | Biome::DeepLukewarmOcean
        | Biome::WarmOcean => plain(sand, sand, 3, Ground::Sand),
        Biome::SnowyBeach => Layers {
            snow_cover: true,
            ..plain(sand, sand, 3, Ground::Sand)
        },
        Biome::SnowyPlains | Biome::SnowyTaiga | Biome::FrozenRiver => grass(true),
        Biome::WindsweptHills if height >= sea_level + 65 => plain(
            palette[BlockKind::Stone],
            palette[BlockKind::Stone],
            0,
            Ground::Bare,
        ),
        Biome::SnowySlopes => plain(
            palette[BlockKind::SnowBlock],
            palette[BlockKind::SnowBlock],
            2,
            Ground::Bare,
        ),
        Biome::FrozenPeaks => plain(
            palette[BlockKind::SnowBlock],
            palette[BlockKind::Stone],
            0,
            Ground::Bare,
        ),
        Biome::JaggedPeaks if height >= sea_level + 127 => plain(
            palette[BlockKind::SnowBlock],
            palette[BlockKind::Stone],
            0,
            Ground::Bare,
        ),
        Biome::JaggedPeaks | Biome::StonyPeaks => plain(
            palette[BlockKind::Stone],
            palette[BlockKind::Stone],
            0,
            Ground::Bare,
        ),
        _ => grass(false),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn underwater_columns_use_seabed_material() {
        let palette = Palette::resolve();
        let layers = layers(&palette, Biome::Ocean, 40, 63, 0.0, 0);
        assert_eq!(layers.ground, Ground::Bare);
        assert_eq!(layers.top, layers.filler);
        assert_ne!(layers.top, palette[BlockKind::GrassBlock]);
    }

    #[test]
    fn warm_seabed_is_mostly_sand() {
        let palette = Palette::resolve();
        let sandy = (-10..=10)
            .filter(|i| {
                layers(&palette, Biome::WarmOcean, 40, 63, f64::from(*i) / 10.0, 0).top
                    == palette[BlockKind::Sand]
            })
            .count();
        assert!(sandy >= 12);
    }

    #[test]
    fn land_biomes_pick_expected_ground() {
        let palette = Palette::resolve();
        let at = |biome| layers(&palette, biome, 70, 63, 0.0, 1);
        assert_eq!(at(Biome::Plains).ground, Ground::Grass);
        assert_eq!(at(Biome::Plains).top, palette[BlockKind::GrassBlock]);
        assert_eq!(at(Biome::SnowyPlains).ground, Ground::SnowyGrass);
        assert!(at(Biome::SnowyPlains).snow_cover);
        assert_eq!(at(Biome::Desert).top, palette[BlockKind::Sand]);
        assert_eq!(at(Biome::Desert).base, Some(palette[BlockKind::Sandstone]));
        assert_eq!(at(Biome::Beach).ground, Ground::Sand);
        assert!(at(Biome::SnowyBeach).snow_cover);
        assert_eq!(at(Biome::StonyPeaks).top, palette[BlockKind::Stone]);
        assert_eq!(at(Biome::FrozenPeaks).top, palette[BlockKind::SnowBlock]);
    }

    #[test]
    fn filler_depth_varies_between_three_and_five() {
        let palette = Palette::resolve();
        for roll in 0..9 {
            let depth = layers(&palette, Biome::Plains, 70, 63, 0.0, roll).filler_depth;
            assert!((3..=5).contains(&depth));
        }
    }

    #[test]
    fn high_hills_lose_grass() {
        let palette = Palette::resolve();
        let low = layers(&palette, Biome::WindsweptHills, 110, 63, 0.0, 0);
        let high = layers(&palette, Biome::WindsweptHills, 140, 63, 0.0, 0);
        assert_eq!(low.ground, Ground::Grass);
        assert_eq!(high.ground, Ground::Bare);
    }
}
