use super::paletted::bits_for_palette;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum HeightmapKind {
    WorldSurfaceWg,
    WorldSurface,
    OceanFloorWg,
    OceanFloor,
    MotionBlocking,
    MotionBlockingNoLeaves,
}

impl HeightmapKind {
    pub const ALL: [Self; 6] = [
        Self::WorldSurfaceWg,
        Self::WorldSurface,
        Self::OceanFloorWg,
        Self::OceanFloor,
        Self::MotionBlocking,
        Self::MotionBlockingNoLeaves,
    ];

    pub const CLIENT: [Self; 3] = [
        Self::WorldSurface,
        Self::MotionBlocking,
        Self::MotionBlockingNoLeaves,
    ];

    pub const fn serialized_name(self) -> &'static str {
        match self {
            Self::WorldSurfaceWg => "WORLD_SURFACE_WG",
            Self::WorldSurface => "WORLD_SURFACE",
            Self::OceanFloorWg => "OCEAN_FLOOR_WG",
            Self::OceanFloor => "OCEAN_FLOOR",
            Self::MotionBlocking => "MOTION_BLOCKING",
            Self::MotionBlockingNoLeaves => "MOTION_BLOCKING_NO_LEAVES",
        }
    }

    pub fn from_serialized_name(name: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|k| k.serialized_name() == name)
    }

    pub const fn protocol_id(self) -> i32 {
        match self {
            Self::WorldSurfaceWg => 0,
            Self::WorldSurface => 1,
            Self::OceanFloorWg => 2,
            Self::OceanFloor => 3,
            Self::MotionBlocking => 4,
            Self::MotionBlockingNoLeaves => 5,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Heightmap {
    heights: Box<[u16; 256]>,
}

impl Default for Heightmap {
    fn default() -> Self {
        Self {
            heights: Box::new([0; 256]),
        }
    }
}

impl Heightmap {
    pub fn get(&self, x: usize, z: usize) -> u16 {
        self.heights[(z << 4) | x]
    }

    pub fn set(&mut self, x: usize, z: usize, height: u16) {
        self.heights[(z << 4) | x] = height;
    }

    pub fn bits(world_height: u32) -> u8 {
        bits_for_palette(world_height as usize + 1, 1)
    }

    pub fn pack(&self, world_height: u32) -> Vec<u64> {
        let bits = Self::bits(world_height) as usize;
        let per_long = 64 / bits;
        let mut out = vec![0u64; 256usize.div_ceil(per_long)];
        for (i, h) in self.heights.iter().enumerate() {
            out[i / per_long] |= u64::from(*h) << ((i % per_long) * bits);
        }
        out
    }

    pub fn unpack(world_height: u32, data: &[u64]) -> Option<Self> {
        let bits = Self::bits(world_height) as usize;
        let per_long = 64 / bits;
        if data.len() != 256usize.div_ceil(per_long) {
            return None;
        }
        let mask = (1u64 << bits) - 1;
        let mut heightmap = Self::default();
        for i in 0..256 {
            heightmap.heights[i] = ((data[i / per_long] >> ((i % per_long) * bits)) & mask) as u16;
        }
        Some(heightmap)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Heightmaps {
    pub maps: Vec<(HeightmapKind, Heightmap)>,
}

impl Heightmaps {
    pub fn get(&self, kind: HeightmapKind) -> Option<&Heightmap> {
        self.maps.iter().find(|(k, _)| *k == kind).map(|(_, h)| h)
    }

    pub fn get_or_insert(&mut self, kind: HeightmapKind) -> &mut Heightmap {
        let index = match self.maps.iter().position(|(k, _)| *k == kind) {
            Some(index) => index,
            None => {
                self.maps.push((kind, Heightmap::default()));
                self.maps.len() - 1
            }
        };
        &mut self.maps[index].1
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pack_roundtrip_for_overworld_height() {
        let mut h = Heightmap::default();
        h.set(3, 7, 200);
        h.set(15, 15, 384);
        let packed = h.pack(384);
        assert_eq!(Heightmap::bits(384), 9);
        assert_eq!(packed.len(), 37);
        assert_eq!(Heightmap::unpack(384, &packed), Some(h));
    }
}
