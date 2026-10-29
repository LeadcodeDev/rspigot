use super::paletted::{Biomes, Blocks, PalettedContainer};
use crate::ids::{BiomeId, BlockStateId};

pub const SECTION_VOLUME: usize = 4096;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ChunkSection {
    blocks: PalettedContainer<BlockStateId, Blocks>,
    biomes: PalettedContainer<BiomeId, Biomes>,
    non_air: u16,
}

pub const fn block_index(x: usize, y: usize, z: usize) -> usize {
    (y << 8) | (z << 4) | x
}

pub const fn biome_index(x: usize, y: usize, z: usize) -> usize {
    (y << 4) | (z << 2) | x
}

impl ChunkSection {
    pub fn empty(biome: BiomeId) -> Self {
        Self {
            blocks: PalettedContainer::single(BlockStateId::AIR),
            biomes: PalettedContainer::single(biome),
            non_air: 0,
        }
    }

    pub fn from_parts(
        blocks: PalettedContainer<BlockStateId, Blocks>,
        biomes: PalettedContainer<BiomeId, Biomes>,
    ) -> Self {
        let non_air = count_non_air(&blocks);
        Self {
            blocks,
            biomes,
            non_air,
        }
    }

    pub fn block(&self, x: usize, y: usize, z: usize) -> BlockStateId {
        self.blocks.get(block_index(x, y, z))
    }

    pub fn set_block(&mut self, x: usize, y: usize, z: usize, state: BlockStateId) -> BlockStateId {
        let old = self.blocks.set(block_index(x, y, z), state);
        match (old.is_air(), state.is_air()) {
            (true, false) => self.non_air += 1,
            (false, true) => self.non_air -= 1,
            _ => {}
        }
        old
    }

    pub fn fill(&mut self, state: BlockStateId) {
        self.blocks.fill(state);
        self.non_air = if state.is_air() {
            0
        } else {
            SECTION_VOLUME as u16
        };
    }

    pub fn biome(&self, x: usize, y: usize, z: usize) -> BiomeId {
        self.biomes.get(biome_index(x, y, z))
    }

    pub fn set_biome(&mut self, x: usize, y: usize, z: usize, biome: BiomeId) {
        self.biomes.set(biome_index(x, y, z), biome);
    }

    pub fn fill_biome(&mut self, biome: BiomeId) {
        self.biomes.fill(biome);
    }

    pub fn blocks(&self) -> &PalettedContainer<BlockStateId, Blocks> {
        &self.blocks
    }

    pub fn biomes(&self) -> &PalettedContainer<BiomeId, Biomes> {
        &self.biomes
    }

    pub fn blocks_mut(&mut self) -> &mut PalettedContainer<BlockStateId, Blocks> {
        &mut self.blocks
    }

    pub fn non_air_count(&self) -> u16 {
        self.non_air
    }

    pub fn is_empty(&self) -> bool {
        self.non_air == 0
    }

    pub fn recount(&mut self) {
        self.non_air = count_non_air(&self.blocks);
    }
}

fn count_non_air(blocks: &PalettedContainer<BlockStateId, Blocks>) -> u16 {
    if let Some(single) = blocks.single_value() {
        return if single.is_air() {
            0
        } else {
            SECTION_VOLUME as u16
        };
    }
    let air_slots: Vec<bool> = blocks.palette().iter().map(|s| s.is_air()).collect();
    (0..SECTION_VOLUME)
        .filter(|i| !air_slots[blocks.palette_index(*i)])
        .count() as u16
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn non_air_count_tracks_changes() {
        let mut s = ChunkSection::empty(BiomeId(0));
        s.set_block(1, 2, 3, BlockStateId(1));
        s.set_block(1, 2, 4, BlockStateId(1));
        assert_eq!(s.non_air_count(), 2);
        s.set_block(1, 2, 3, BlockStateId::AIR);
        assert_eq!(s.non_air_count(), 1);
        assert_eq!(s.block(1, 2, 4), BlockStateId(1));
    }

    #[test]
    fn fill_sets_full_count() {
        let mut s = ChunkSection::empty(BiomeId(0));
        s.fill(BlockStateId(1));
        assert_eq!(s.non_air_count(), 4096);
    }
}
