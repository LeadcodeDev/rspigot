use std::borrow::Cow;

use rspigot_nbt::Tag;

use crate::Packet;
use crate::codec::{
    Decode, DecodeError, Encode, MAX_LIST_LEN, NetworkNbt, decode_len, encode_len, take,
};
use crate::ids::play::clientbound as id;
use crate::play::types::{ByteBitSet, ChunkPos};
use crate::play::wire::{read_var, wire_enum, write_var};

pub const SECTION_WIDTH: usize = 16;
pub const BLOCKS_PER_SECTION: usize = 4096;
pub const BIOMES_PER_SECTION: usize = 64;
pub const HEIGHTMAP_ENTRIES: usize = 256;
pub const LIGHT_ARRAY_BYTES: usize = 2048;
pub const MAX_CHUNK_DATA_BYTES: usize = 2_097_152;
pub const VANILLA_BLOCK_STATE_COUNT: u32 = 35_723;
pub const VANILLA_BIOME_COUNT: u32 = 67;
pub const BLOCK_MIN_INDIRECT_BITS: u8 = 4;
pub const BLOCK_MAX_INDIRECT_BITS: u8 = 8;
pub const BIOME_MIN_INDIRECT_BITS: u8 = 1;
pub const BIOME_MAX_INDIRECT_BITS: u8 = 3;

pub type LightArray = [u8; LIGHT_ARRAY_BYTES];

pub const fn ceil_log2(value: u32) -> u8 {
    if value <= 1 {
        0
    } else {
        (32 - (value - 1).leading_zeros()) as u8
    }
}

pub const fn heightmap_bits(world_height: u32) -> u8 {
    ceil_log2(world_height + 1)
}

pub fn pack_entries(bits: u8, values: &[u32]) -> Vec<u64> {
    if bits == 0 {
        return Vec::new();
    }
    let bits = usize::from(bits);
    let per_long = 64 / bits;
    let mask = if bits >= 64 {
        u64::MAX
    } else {
        (1u64 << bits) - 1
    };
    let mut packed = vec![0u64; values.len().div_ceil(per_long)];
    for (index, value) in values.iter().enumerate() {
        packed[index / per_long] |= (u64::from(*value) & mask) << ((index % per_long) * bits);
    }
    packed
}

pub fn unpack_entries(bits: u8, count: usize, packed: &[u64]) -> Vec<u32> {
    if bits == 0 {
        return vec![0; count];
    }
    let bits = usize::from(bits);
    let per_long = 64 / bits;
    let mask = if bits >= 64 {
        u64::MAX
    } else {
        (1u64 << bits) - 1
    };
    (0..count)
        .map(|index| {
            packed.get(index / per_long).map_or(0, |word| {
                ((word >> ((index % per_long) * bits)) & mask) as u32
            })
        })
        .collect()
}

pub fn pack_heightmap(world_height: u32, heights: &[u32]) -> Vec<i64> {
    pack_entries(heightmap_bits(world_height), heights)
        .into_iter()
        .map(|word| word as i64)
        .collect()
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PaletteStrategy {
    pub entries: usize,
    pub min_indirect_bits: u8,
    pub max_indirect_bits: u8,
    pub direct_bits: u8,
}

impl PaletteStrategy {
    pub const VANILLA_BLOCKS: Self = Self::blocks(VANILLA_BLOCK_STATE_COUNT);
    pub const VANILLA_BIOMES: Self = Self::biomes(VANILLA_BIOME_COUNT);

    pub const fn blocks(registry_size: u32) -> Self {
        Self {
            entries: BLOCKS_PER_SECTION,
            min_indirect_bits: BLOCK_MIN_INDIRECT_BITS,
            max_indirect_bits: BLOCK_MAX_INDIRECT_BITS,
            direct_bits: ceil_log2(registry_size),
        }
    }

    pub const fn biomes(registry_size: u32) -> Self {
        Self {
            entries: BIOMES_PER_SECTION,
            min_indirect_bits: BIOME_MIN_INDIRECT_BITS,
            max_indirect_bits: BIOME_MAX_INDIRECT_BITS,
            direct_bits: ceil_log2(registry_size),
        }
    }

    pub const fn storage_bits(&self, wire_bits: u8) -> u8 {
        if wire_bits == 0 {
            0
        } else if wire_bits <= self.min_indirect_bits {
            self.min_indirect_bits
        } else if wire_bits <= self.max_indirect_bits {
            wire_bits
        } else {
            self.direct_bits
        }
    }

    pub const fn bits_for_palette_len(&self, palette_len: usize) -> u8 {
        if palette_len <= 1 {
            0
        } else {
            self.storage_bits(ceil_log2(palette_len as u32))
        }
    }

    pub const fn is_direct(&self, bits: u8) -> bool {
        bits > self.max_indirect_bits
    }

    pub const fn long_count(&self, bits: u8) -> usize {
        if bits == 0 {
            0
        } else {
            self.entries.div_ceil(64 / bits as usize)
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PalettedContainer<'a> {
    Single(u32),
    Indirect {
        bits: u8,
        palette: Cow<'a, [u32]>,
        packed: Cow<'a, [u64]>,
    },
    Direct {
        bits: u8,
        packed: Cow<'a, [u64]>,
    },
}

impl<'a> PalettedContainer<'a> {
    pub const fn single(value: u32) -> Self {
        Self::Single(value)
    }

    pub const fn indirect(bits: u8, palette: &'a [u32], packed: &'a [u64]) -> Self {
        Self::Indirect {
            bits,
            palette: Cow::Borrowed(palette),
            packed: Cow::Borrowed(packed),
        }
    }

    pub const fn direct(bits: u8, packed: &'a [u64]) -> Self {
        Self::Direct {
            bits,
            packed: Cow::Borrowed(packed),
        }
    }

    pub fn from_entries(strategy: &PaletteStrategy, entries: &[u32]) -> PalettedContainer<'static> {
        let mut palette: Vec<u32> = Vec::new();
        let mut indices = Vec::with_capacity(entries.len());
        for value in entries {
            let index = match palette.iter().position(|known| known == value) {
                Some(index) => index,
                None => {
                    palette.push(*value);
                    palette.len() - 1
                }
            };
            indices.push(index as u32);
        }
        let bits = strategy.bits_for_palette_len(palette.len());
        if bits == 0 {
            return PalettedContainer::Single(palette.first().copied().unwrap_or(0));
        }
        if strategy.is_direct(bits) {
            return PalettedContainer::Direct {
                bits,
                packed: Cow::Owned(pack_entries(bits, entries)),
            };
        }
        PalettedContainer::Indirect {
            bits,
            packed: Cow::Owned(pack_entries(bits, &indices)),
            palette: Cow::Owned(palette),
        }
    }

    pub const fn bits(&self) -> u8 {
        match self {
            Self::Single(_) => 0,
            Self::Indirect { bits, .. } | Self::Direct { bits, .. } => *bits,
        }
    }

    pub fn entries(&self, strategy: &PaletteStrategy) -> Vec<u32> {
        match self {
            Self::Single(value) => vec![*value; strategy.entries],
            Self::Indirect {
                bits,
                palette,
                packed,
            } => unpack_entries(*bits, strategy.entries, packed)
                .into_iter()
                .map(|index| palette.get(index as usize).copied().unwrap_or(0))
                .collect(),
            Self::Direct { bits, packed } => unpack_entries(*bits, strategy.entries, packed),
        }
    }

    pub fn validate(&self, strategy: &PaletteStrategy) -> Result<(), DecodeError> {
        let (bits, packed_len, palette_len) = match self {
            Self::Single(_) => return Ok(()),
            Self::Indirect {
                bits,
                palette,
                packed,
            } => {
                if strategy.is_direct(*bits) {
                    return Err(invalid("indirect palette bits", *bits));
                }
                (*bits, packed.len(), Some(palette.len()))
            }
            Self::Direct { bits, packed } => {
                if *bits != strategy.direct_bits {
                    return Err(invalid("direct palette bits", *bits));
                }
                (*bits, packed.len(), None)
            }
        };
        if strategy.storage_bits(bits) != bits {
            return Err(invalid("palette storage bits", bits));
        }
        if let Some(len) = palette_len
            && len > 1usize << bits
        {
            return Err(DecodeError::TooLong {
                len,
                max: 1 << bits,
            });
        }
        let expected = strategy.long_count(bits);
        if packed_len != expected {
            return Err(DecodeError::InvalidValue {
                field: "packed long count",
                value: packed_len as i64,
            });
        }
        Ok(())
    }

    pub fn into_owned(self) -> PalettedContainer<'static> {
        match self {
            Self::Single(value) => PalettedContainer::Single(value),
            Self::Indirect {
                bits,
                palette,
                packed,
            } => PalettedContainer::Indirect {
                bits,
                palette: Cow::Owned(palette.into_owned()),
                packed: Cow::Owned(packed.into_owned()),
            },
            Self::Direct { bits, packed } => PalettedContainer::Direct {
                bits,
                packed: Cow::Owned(packed.into_owned()),
            },
        }
    }

    pub fn decode(
        input: &mut &[u8],
        strategy: &PaletteStrategy,
    ) -> Result<PalettedContainer<'static>, DecodeError> {
        let wire_bits = u8::decode(input)?;
        let bits = strategy.storage_bits(wire_bits);
        if bits == 0 {
            return Ok(PalettedContainer::Single(read_var(input)? as u32));
        }
        if strategy.is_direct(bits) {
            let packed = read_longs(input, strategy.long_count(bits))?;
            return Ok(PalettedContainer::Direct {
                bits,
                packed: Cow::Owned(packed),
            });
        }
        let len = decode_len(input, 1 << bits)?;
        let mut palette = Vec::with_capacity(len);
        for _ in 0..len {
            palette.push(read_var(input)? as u32);
        }
        let packed = read_longs(input, strategy.long_count(bits))?;
        Ok(PalettedContainer::Indirect {
            bits,
            palette: Cow::Owned(palette),
            packed: Cow::Owned(packed),
        })
    }
}

fn invalid(field: &'static str, bits: u8) -> DecodeError {
    DecodeError::InvalidValue {
        field,
        value: i64::from(bits),
    }
}

fn write_longs(packed: &[u64], out: &mut Vec<u8>) {
    out.reserve(packed.len() * 8);
    for word in packed {
        out.extend_from_slice(&word.to_be_bytes());
    }
}

fn read_longs(input: &mut &[u8], count: usize) -> Result<Vec<u64>, DecodeError> {
    let bytes = take(input, count * 8)?;
    Ok(bytes
        .as_chunks::<8>()
        .0
        .iter()
        .map(|chunk| u64::from_be_bytes(*chunk))
        .collect())
}

impl Encode for PalettedContainer<'_> {
    fn encode(&self, out: &mut Vec<u8>) {
        match self {
            Self::Single(value) => {
                out.push(0);
                write_var(*value as i32, out);
            }
            Self::Indirect {
                bits,
                palette,
                packed,
            } => {
                out.push(*bits);
                encode_len(palette.len(), out);
                for value in palette.iter() {
                    write_var(*value as i32, out);
                }
                write_longs(packed, out);
            }
            Self::Direct { bits, packed } => {
                out.push(*bits);
                write_longs(packed, out);
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ChunkSection<'a> {
    pub non_empty_block_count: i16,
    pub fluid_count: i16,
    pub blocks: PalettedContainer<'a>,
    pub biomes: PalettedContainer<'a>,
}

impl ChunkSection<'static> {
    pub const fn uniform(
        block_state: u32,
        biome: u32,
        non_empty_block_count: i16,
        fluid_count: i16,
    ) -> Self {
        Self {
            non_empty_block_count,
            fluid_count,
            blocks: PalettedContainer::Single(block_state),
            biomes: PalettedContainer::Single(biome),
        }
    }
}

impl ChunkSection<'_> {
    pub fn validate(
        &self,
        blocks: &PaletteStrategy,
        biomes: &PaletteStrategy,
    ) -> Result<(), DecodeError> {
        self.blocks.validate(blocks)?;
        self.biomes.validate(biomes)
    }

    pub fn decode(
        input: &mut &[u8],
        blocks: &PaletteStrategy,
        biomes: &PaletteStrategy,
    ) -> Result<ChunkSection<'static>, DecodeError> {
        Ok(ChunkSection {
            non_empty_block_count: i16::decode(input)?,
            fluid_count: i16::decode(input)?,
            blocks: PalettedContainer::decode(input, blocks)?,
            biomes: PalettedContainer::decode(input, biomes)?,
        })
    }
}

impl Encode for ChunkSection<'_> {
    fn encode(&self, out: &mut Vec<u8>) {
        self.non_empty_block_count.encode(out);
        self.fluid_count.encode(out);
        self.blocks.encode(out);
        self.biomes.encode(out);
    }
}

pub fn write_sections(sections: &[ChunkSection<'_>], out: &mut Vec<u8>) {
    for section in sections {
        section.encode(out);
    }
}

pub fn decode_sections(
    mut data: &[u8],
    count: usize,
    blocks: &PaletteStrategy,
    biomes: &PaletteStrategy,
) -> Result<Vec<ChunkSection<'static>>, DecodeError> {
    let mut sections = Vec::with_capacity(count);
    for _ in 0..count {
        sections.push(ChunkSection::decode(&mut data, blocks, biomes)?);
    }
    if data.is_empty() {
        Ok(sections)
    } else {
        Err(DecodeError::TrailingBytes(data.len()))
    }
}

wire_enum! {
    pub enum HeightmapKind: Zero {
        WorldSurfaceWorldgen,
        WorldSurface,
        OceanFloorWorldgen,
        OceanFloor,
        MotionBlocking,
        MotionBlockingNoLeaves,
    }
}

impl HeightmapKind {
    pub const SENT_TO_CLIENT: [Self; 3] = [
        Self::WorldSurface,
        Self::MotionBlocking,
        Self::MotionBlockingNoLeaves,
    ];
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Heightmap<'a> {
    pub kind: HeightmapKind,
    pub data: Cow<'a, [i64]>,
}

impl Encode for Heightmap<'_> {
    fn encode(&self, out: &mut Vec<u8>) {
        self.kind.encode(out);
        encode_len(self.data.len(), out);
        for word in self.data.iter() {
            word.encode(out);
        }
    }
}

impl Decode for Heightmap<'static> {
    fn decode(input: &mut &[u8]) -> Result<Self, DecodeError> {
        let kind = HeightmapKind::decode(input)?;
        let len = decode_len(input, MAX_LIST_LEN)?;
        let words = read_longs(input, len)?;
        Ok(Self {
            kind,
            data: Cow::Owned(words.into_iter().map(|w| w as i64).collect()),
        })
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BlockEntityInfo {
    pub packed_xz: u8,
    pub y: i16,
    pub kind: i32,
    pub data: Option<Tag>,
}

impl BlockEntityInfo {
    pub const fn pack_xz(local_x: u8, local_z: u8) -> u8 {
        ((local_x & 15) << 4) | (local_z & 15)
    }

    pub const fn local_x(&self) -> u8 {
        self.packed_xz >> 4
    }

    pub const fn local_z(&self) -> u8 {
        self.packed_xz & 15
    }
}

impl Encode for BlockEntityInfo {
    fn encode(&self, out: &mut Vec<u8>) {
        self.packed_xz.encode(out);
        self.y.encode(out);
        write_var(self.kind, out);
        match &self.data {
            Some(tag) => rspigot_nbt::write_network(out, tag),
            None => out.push(0),
        }
    }
}

impl Decode for BlockEntityInfo {
    fn decode(input: &mut &[u8]) -> Result<Self, DecodeError> {
        let packed_xz = u8::decode(input)?;
        let y = i16::decode(input)?;
        let kind = read_var(input)?;
        let data = NetworkNbt::decode(input)?.0;
        if let Some(tag) = &data
            && !matches!(tag, Tag::Compound(_))
        {
            return Err(DecodeError::InvalidValue {
                field: "block entity tag",
                value: i64::from(tag.tag_type() as u8),
            });
        }
        Ok(Self {
            packed_xz,
            y,
            kind,
            data,
        })
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LevelChunkData<'a> {
    pub heightmaps: Vec<Heightmap<'a>>,
    pub data: Cow<'a, [u8]>,
    pub block_entities: Cow<'a, [BlockEntityInfo]>,
}

impl Encode for LevelChunkData<'_> {
    fn encode(&self, out: &mut Vec<u8>) {
        self.heightmaps.encode(out);
        encode_len(self.data.len(), out);
        out.extend_from_slice(&self.data);
        self.block_entities.as_ref().encode(out);
    }
}

impl Decode for LevelChunkData<'static> {
    fn decode(input: &mut &[u8]) -> Result<Self, DecodeError> {
        let heightmaps = Vec::<Heightmap<'static>>::decode(input)?;
        let len = decode_len(input, MAX_CHUNK_DATA_BYTES)?;
        let data = take(input, len)?.to_vec();
        let block_entities = Vec::<BlockEntityInfo>::decode(input)?;
        Ok(Self {
            heightmaps,
            data: Cow::Owned(data),
            block_entities: Cow::Owned(block_entities),
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct LightData<'a> {
    pub sky_y_mask: ByteBitSet,
    pub block_y_mask: ByteBitSet,
    pub empty_sky_y_mask: ByteBitSet,
    pub empty_block_y_mask: ByteBitSet,
    pub sky_updates: Vec<Cow<'a, LightArray>>,
    pub block_updates: Vec<Cow<'a, LightArray>>,
}

fn write_light_arrays(arrays: &[Cow<'_, LightArray>], out: &mut Vec<u8>) {
    encode_len(arrays.len(), out);
    for array in arrays {
        encode_len(LIGHT_ARRAY_BYTES, out);
        out.extend_from_slice(array.as_ref());
    }
}

fn read_light_arrays(input: &mut &[u8]) -> Result<Vec<Cow<'static, LightArray>>, DecodeError> {
    let count = decode_len(input, MAX_LIST_LEN)?;
    let mut arrays = Vec::with_capacity(count.min(input.len() / LIGHT_ARRAY_BYTES));
    for _ in 0..count {
        let len = decode_len(input, LIGHT_ARRAY_BYTES)?;
        if len != LIGHT_ARRAY_BYTES {
            return Err(DecodeError::InvalidValue {
                field: "light array length",
                value: len as i64,
            });
        }
        let mut array = [0u8; LIGHT_ARRAY_BYTES];
        array.copy_from_slice(take(input, LIGHT_ARRAY_BYTES)?);
        arrays.push(Cow::Owned(array));
    }
    Ok(arrays)
}

impl Encode for LightData<'_> {
    fn encode(&self, out: &mut Vec<u8>) {
        self.sky_y_mask.encode(out);
        self.block_y_mask.encode(out);
        self.empty_sky_y_mask.encode(out);
        self.empty_block_y_mask.encode(out);
        write_light_arrays(&self.sky_updates, out);
        write_light_arrays(&self.block_updates, out);
    }
}

impl Decode for LightData<'static> {
    fn decode(input: &mut &[u8]) -> Result<Self, DecodeError> {
        Ok(Self {
            sky_y_mask: ByteBitSet::decode(input)?,
            block_y_mask: ByteBitSet::decode(input)?,
            empty_sky_y_mask: ByteBitSet::decode(input)?,
            empty_block_y_mask: ByteBitSet::decode(input)?,
            sky_updates: read_light_arrays(input)?,
            block_updates: read_light_arrays(input)?,
        })
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LevelChunkWithLight<'a> {
    pub x: i32,
    pub z: i32,
    pub chunk: LevelChunkData<'a>,
    pub light: LightData<'a>,
}

impl Encode for LevelChunkWithLight<'_> {
    fn encode(&self, out: &mut Vec<u8>) {
        self.x.encode(out);
        self.z.encode(out);
        self.chunk.encode(out);
        self.light.encode(out);
    }
}

impl Decode for LevelChunkWithLight<'static> {
    fn decode(input: &mut &[u8]) -> Result<Self, DecodeError> {
        Ok(Self {
            x: i32::decode(input)?,
            z: i32::decode(input)?,
            chunk: LevelChunkData::decode(input)?,
            light: LightData::decode(input)?,
        })
    }
}

impl Packet for LevelChunkWithLight<'_> {
    const ID: i32 = id::LEVEL_CHUNK_WITH_LIGHT;
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LightUpdate<'a> {
    pub x: i32,
    pub z: i32,
    pub light: LightData<'a>,
}

impl Encode for LightUpdate<'_> {
    fn encode(&self, out: &mut Vec<u8>) {
        write_var(self.x, out);
        write_var(self.z, out);
        self.light.encode(out);
    }
}

impl Decode for LightUpdate<'static> {
    fn decode(input: &mut &[u8]) -> Result<Self, DecodeError> {
        Ok(Self {
            x: read_var(input)?,
            z: read_var(input)?,
            light: LightData::decode(input)?,
        })
    }
}

impl Packet for LightUpdate<'_> {
    const ID: i32 = id::LIGHT_UPDATE;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ForgetLevelChunk {
    pub pos: ChunkPos,
}

impl Encode for ForgetLevelChunk {
    fn encode(&self, out: &mut Vec<u8>) {
        self.pos.encode(out);
    }
}

impl Decode for ForgetLevelChunk {
    fn decode(input: &mut &[u8]) -> Result<Self, DecodeError> {
        ChunkPos::decode(input).map(|pos| Self { pos })
    }
}

impl Packet for ForgetLevelChunk {
    const ID: i32 = id::FORGET_LEVEL_CHUNK;
}

#[cfg(test)]
mod tests {
    use rspigot_nbt::Compound;

    use super::*;
    use crate::codec::decode_exact;
    use crate::play::wire::testing::{encoded, roundtrip};

    const BLOCKS: PaletteStrategy = PaletteStrategy::VANILLA_BLOCKS;
    const BIOMES: PaletteStrategy = PaletteStrategy::VANILLA_BIOMES;
    const STONE: u32 = 1;
    const GRASS_BLOCK: u32 = 9;
    const PLAINS: u32 = 40;

    #[test]
    fn vanilla_thresholds() {
        assert_eq!(BLOCKS.direct_bits, 16);
        assert_eq!(BIOMES.direct_bits, 7);
        assert_eq!(BLOCKS.bits_for_palette_len(1), 0);
        assert_eq!(BLOCKS.bits_for_palette_len(2), 4);
        assert_eq!(BLOCKS.bits_for_palette_len(16), 4);
        assert_eq!(BLOCKS.bits_for_palette_len(17), 5);
        assert_eq!(BLOCKS.bits_for_palette_len(256), 8);
        assert_eq!(BLOCKS.bits_for_palette_len(257), 16);
        assert_eq!(BIOMES.bits_for_palette_len(2), 1);
        assert_eq!(BIOMES.bits_for_palette_len(3), 2);
        assert_eq!(BIOMES.bits_for_palette_len(8), 3);
        assert_eq!(BIOMES.bits_for_palette_len(9), 7);
        assert_eq!(BLOCKS.storage_bits(2), 4);
        assert_eq!(BLOCKS.storage_bits(12), 16);
        assert_eq!(BLOCKS.long_count(4), 256);
        assert_eq!(BLOCKS.long_count(5), 342);
        assert_eq!(BLOCKS.long_count(16), 1024);
        assert_eq!(BIOMES.long_count(1), 1);
        assert_eq!(BIOMES.long_count(7), 8);
        assert_eq!(heightmap_bits(384), 9);
    }

    #[test]
    fn packing_never_spans_longs() {
        let values: Vec<u32> = (0..64).map(|i| i % 32).collect();
        let packed = pack_entries(5, &values);
        assert_eq!(packed.len(), 6);
        assert_eq!(packed[0] & 0x1F, 0);
        assert_eq!((packed[0] >> 5) & 0x1F, 1);
        assert_eq!(packed[1] & 0x1F, 12);
        assert_eq!(packed[0] >> 60, 0);
        assert_eq!(unpack_entries(5, 64, &packed), values);
    }

    #[test]
    fn single_value_section_golden() {
        let section = ChunkSection::uniform(STONE, PLAINS, 4096, 0);
        assert_eq!(
            encoded(&section),
            vec![0x10, 0x00, 0x00, 0x00, 0x00, 0x01, 0x00, 0x28]
        );
        let air = ChunkSection::uniform(0, 0, 0, 0);
        assert_eq!(encoded(&air), vec![0; 8]);
    }

    #[test]
    fn indirect_section_layout() {
        let mut entries = vec![STONE; BLOCKS_PER_SECTION];
        entries[1] = GRASS_BLOCK;
        let blocks = PalettedContainer::from_entries(&BLOCKS, &entries);
        assert_eq!(blocks.bits(), 4);
        blocks.validate(&BLOCKS).unwrap();
        let mut biome_entries = vec![PLAINS; BIOMES_PER_SECTION];
        biome_entries[63] = 1;
        let biomes = PalettedContainer::from_entries(&BIOMES, &biome_entries);
        assert_eq!(biomes.bits(), 1);
        let section = ChunkSection {
            non_empty_block_count: 4096,
            fluid_count: 0,
            blocks,
            biomes,
        };
        let bytes = encoded(&section);
        assert_eq!(&bytes[..8], &[0x10, 0x00, 0x00, 0x00, 4, 2, 1, 9]);
        assert_eq!(&bytes[8..15], &[0, 0, 0, 0, 0, 0, 0]);
        assert_eq!(bytes[15], 0x10);
        assert_eq!(bytes.len(), 4 + 1 + 3 + 256 * 8 + 1 + 3 + 8);
        let biome_long = &bytes[bytes.len() - 8..];
        assert_eq!(biome_long, &[0x80, 0, 0, 0, 0, 0, 0, 0]);
        let mut input = bytes.as_slice();
        let decoded = ChunkSection::decode(&mut input, &BLOCKS, &BIOMES).unwrap();
        assert!(input.is_empty());
        assert_eq!(decoded, section);
        assert_eq!(decoded.blocks.entries(&BLOCKS), entries);
        assert_eq!(decoded.biomes.entries(&BIOMES), biome_entries);
    }

    #[test]
    fn direct_palette_uses_registry_bits() {
        let entries: Vec<u32> = (0..BLOCKS_PER_SECTION as u32).collect();
        let blocks = PalettedContainer::from_entries(&BLOCKS, &entries);
        assert!(matches!(blocks, PalettedContainer::Direct { bits: 16, .. }));
        blocks.validate(&BLOCKS).unwrap();
        let bytes = encoded(&blocks);
        assert_eq!(bytes[0], 16);
        assert_eq!(bytes.len(), 1 + 1024 * 8);
        let decoded = PalettedContainer::decode(&mut bytes.as_slice(), &BLOCKS).unwrap();
        assert_eq!(decoded.entries(&BLOCKS), entries);
    }

    #[test]
    fn validation_rejects_bits_the_client_would_misread() {
        let packed = vec![0u64; 128];
        let palette = [STONE, GRASS_BLOCK];
        assert!(
            PalettedContainer::indirect(2, &palette, &packed)
                .validate(&BLOCKS)
                .is_err()
        );
        let packed = vec![0u64; 256];
        PalettedContainer::indirect(4, &palette, &packed)
            .validate(&BLOCKS)
            .unwrap();
        assert!(
            PalettedContainer::indirect(4, &palette, &packed[..255])
                .validate(&BLOCKS)
                .is_err()
        );
        assert!(
            PalettedContainer::direct(15, &[0; 1024])
                .validate(&BLOCKS)
                .is_err()
        );
    }

    #[test]
    fn decode_follows_client_storage_bits() {
        let mut bytes = vec![2, 1, 1];
        bytes.extend(std::iter::repeat_n(0, 256 * 8));
        let decoded = PalettedContainer::decode(&mut bytes.as_slice(), &BLOCKS).unwrap();
        assert_eq!(decoded.bits(), 4);
    }

    fn overworld_column() -> (Vec<u8>, LevelChunkWithLight<'static>) {
        let mut sections = Vec::new();
        for index in 0..24 {
            let block = if index < 4 { STONE } else { 0 };
            let count = if index < 4 { 4096 } else { 0 };
            sections.push(ChunkSection::uniform(block, PLAINS, count, 0));
        }
        let mut data = Vec::new();
        write_sections(&sections, &mut data);
        let heights = vec![64u32 + 64; HEIGHTMAP_ENTRIES];
        let packed = pack_heightmap(384, &heights);
        let heightmaps = HeightmapKind::SENT_TO_CLIENT
            .iter()
            .map(|kind| Heightmap {
                kind: *kind,
                data: Cow::Owned(packed.clone()),
            })
            .collect();
        let full_bright: LightArray = [0xFF; LIGHT_ARRAY_BYTES];
        let packet = LevelChunkWithLight {
            x: -2,
            z: 5,
            chunk: LevelChunkData {
                heightmaps,
                data: Cow::Owned(data.clone()),
                block_entities: Cow::Owned(vec![BlockEntityInfo {
                    packed_xz: BlockEntityInfo::pack_xz(3, 12),
                    y: -60,
                    kind: 7,
                    data: Some(Tag::Compound(Compound::new().with("id", "minecraft:chest"))),
                }]),
            },
            light: LightData {
                sky_y_mask: ByteBitSet::from_bits(25..26),
                block_y_mask: ByteBitSet::default(),
                empty_sky_y_mask: ByteBitSet::from_bits(0..25),
                empty_block_y_mask: ByteBitSet::from_bits(0..26),
                sky_updates: vec![Cow::Owned(full_bright)],
                block_updates: Vec::new(),
            },
        };
        (data, packet)
    }

    #[test]
    fn full_column_of_24_single_value_sections() {
        let (data, packet) = overworld_column();
        assert_eq!(data.len(), 24 * 8);
        assert_eq!(
            &data[..8],
            &[0x10, 0x00, 0x00, 0x00, 0x00, 0x01, 0x00, 0x28]
        );
        assert_eq!(
            &data[32..40],
            &[0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x28]
        );
        let bytes = roundtrip(&packet);
        assert_eq!(&bytes[..8], &[0xFF, 0xFF, 0xFF, 0xFE, 0, 0, 0, 5]);
        assert_eq!(bytes[8], 3);
        assert_eq!(bytes[9], 1);
        assert_eq!(bytes[10], 37);
        let heightmap_bytes = 3 * (1 + 1 + 37 * 8);
        let data_start = 8 + 1 + heightmap_bytes;
        assert_eq!(&bytes[data_start..data_start + 2], &[0xC0, 0x01]);
        assert_eq!(
            &bytes[data_start + 2..data_start + 2 + data.len()],
            data.as_slice()
        );
        let decoded = decode_exact::<LevelChunkWithLight<'static>>(&bytes).unwrap();
        let sections = decode_sections(&decoded.chunk.data, 24, &BLOCKS, &BIOMES).unwrap();
        assert_eq!(sections.len(), 24);
        assert!(
            sections
                .iter()
                .all(|s| s.biomes == PalettedContainer::Single(PLAINS))
        );
        assert_eq!(sections[3].blocks, PalettedContainer::Single(STONE));
        assert_eq!(sections[4].non_empty_block_count, 0);
        let light_start = bytes.len() - (5 + 1 + 5 + 5 + 1 + 2 + LIGHT_ARRAY_BYTES + 1);
        assert_eq!(
            &bytes[light_start..light_start + 17],
            &[
                4, 0, 0, 0, 0x02, 0, 4, 0xFF, 0xFF, 0xFF, 0x01, 4, 0xFF, 0xFF, 0xFF, 0x03, 1
            ]
        );
        assert_eq!(&bytes[light_start + 17..light_start + 19], &[0x80, 0x10]);
        assert_eq!(*bytes.last().unwrap(), 0);
    }

    #[test]
    fn block_entity_rejects_non_compound_tag() {
        let mut bytes = vec![0x3C, 0xFF, 0xC4, 7];
        rspigot_nbt::write_network(&mut bytes, &Tag::Int(1));
        assert!(decode_exact::<BlockEntityInfo>(&bytes).is_err());
        let empty = BlockEntityInfo {
            packed_xz: 0x3C,
            y: -60,
            kind: 7,
            data: None,
        };
        assert_eq!(roundtrip(&empty), vec![0x3C, 0xFF, 0xC4, 7, 0]);
        assert_eq!(empty.local_x(), 3);
        assert_eq!(empty.local_z(), 12);
    }

    #[test]
    fn light_update_and_forget_chunk() {
        let packet = LightUpdate {
            x: 300,
            z: -1,
            light: LightData::default(),
        };
        assert_eq!(
            roundtrip(&packet),
            vec![0xAC, 0x02, 0xFF, 0xFF, 0xFF, 0xFF, 0x0F, 0, 0, 0, 0, 0, 0]
        );
        let mut short = vec![0, 0, 0, 0, 1];
        encode_len(LIGHT_ARRAY_BYTES - 1, &mut short);
        short.extend(std::iter::repeat_n(0, LIGHT_ARRAY_BYTES - 1));
        short.push(0);
        assert!(decode_exact::<LightData<'static>>(&short).is_err());
        let forget = ForgetLevelChunk {
            pos: ChunkPos { x: 1, z: 2 },
        };
        assert_eq!(roundtrip(&forget), vec![0, 0, 0, 2, 0, 0, 0, 1]);
    }
}
