use std::marker::PhantomData;

pub trait PaletteStrategy {
    const SIZE: usize;
    const MIN_BITS: u8;
    const MAX_INDIRECT_BITS: u8;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Blocks;

impl PaletteStrategy for Blocks {
    const SIZE: usize = 4096;
    const MIN_BITS: u8 = 4;
    const MAX_INDIRECT_BITS: u8 = 8;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Biomes;

impl PaletteStrategy for Biomes {
    const SIZE: usize = 64;
    const MIN_BITS: u8 = 1;
    const MAX_INDIRECT_BITS: u8 = 3;
}

#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum PaletteError {
    #[error("empty palette")]
    EmptyPalette,
    #[error("palette of {palette} entries does not fit in {bits} bits")]
    PaletteTooLarge { palette: usize, bits: u8 },
    #[error("expected {expected} packed longs, got {actual}")]
    DataLength { expected: usize, actual: usize },
    #[error("palette index {index} out of range {len}")]
    IndexOutOfRange { index: usize, len: usize },
    #[error("unsupported bit width {0}")]
    BitWidth(u8),
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum Storage<T> {
    Single(T),
    Indirect {
        palette: Vec<T>,
        bits: u8,
        data: Box<[u64]>,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PalettedContainer<T, S> {
    storage: Storage<T>,
    strategy: PhantomData<S>,
}

pub const fn packed_len(size: usize, bits: u8) -> usize {
    let per_long = 64 / bits as usize;
    size.div_ceil(per_long)
}

pub const fn bits_for_palette(len: usize, min_bits: u8) -> u8 {
    let mut bits = 0u8;
    while (1usize << bits) < len {
        bits += 1;
    }
    if bits < min_bits { min_bits } else { bits }
}

fn read_index(data: &[u64], bits: u8, index: usize) -> usize {
    let per_long = 64 / bits as usize;
    let mask = (1u64 << bits) - 1;
    let shift = (index % per_long) * bits as usize;
    ((data[index / per_long] >> shift) & mask) as usize
}

fn write_index(data: &mut [u64], bits: u8, index: usize, value: usize) {
    let per_long = 64 / bits as usize;
    let mask = (1u64 << bits) - 1;
    let shift = (index % per_long) * bits as usize;
    let slot = &mut data[index / per_long];
    *slot = (*slot & !(mask << shift)) | ((value as u64 & mask) << shift);
}

fn repack(size: usize, data: &[u64], from: u8, to: u8) -> Box<[u64]> {
    let mut out = vec![0u64; packed_len(size, to)].into_boxed_slice();
    for i in 0..size {
        write_index(&mut out, to, i, read_index(data, from, i));
    }
    out
}

impl<T: Copy + Eq, S: PaletteStrategy> PalettedContainer<T, S> {
    pub fn single(value: T) -> Self {
        Self {
            storage: Storage::Single(value),
            strategy: PhantomData,
        }
    }

    pub fn from_packed(palette: Vec<T>, bits: u8, data: Vec<u64>) -> Result<Self, PaletteError> {
        if palette.is_empty() {
            return Err(PaletteError::EmptyPalette);
        }
        if palette.len() == 1 {
            return Ok(Self::single(palette[0]));
        }
        if bits == 0 || bits > 32 {
            return Err(PaletteError::BitWidth(bits));
        }
        if palette.len() > (1usize << bits) {
            return Err(PaletteError::PaletteTooLarge {
                palette: palette.len(),
                bits,
            });
        }
        let expected = packed_len(S::SIZE, bits);
        if data.len() != expected {
            return Err(PaletteError::DataLength {
                expected,
                actual: data.len(),
            });
        }
        for i in 0..S::SIZE {
            let index = read_index(&data, bits, i);
            if index >= palette.len() {
                return Err(PaletteError::IndexOutOfRange {
                    index,
                    len: palette.len(),
                });
            }
        }
        let data = data.into_boxed_slice();
        let target = bits.max(S::MIN_BITS);
        let data = if target == bits {
            data
        } else {
            repack(S::SIZE, &data, bits, target)
        };
        Ok(Self {
            storage: Storage::Indirect {
                palette,
                bits: target,
                data,
            },
            strategy: PhantomData,
        })
    }

    pub fn from_values(values: &[T]) -> Self {
        assert_eq!(values.len(), S::SIZE);
        let mut container = Self::single(values[0]);
        for (i, v) in values.iter().enumerate().skip(1) {
            container.set(i, *v);
        }
        container
    }

    pub fn get(&self, index: usize) -> T {
        match &self.storage {
            Storage::Single(v) => *v,
            Storage::Indirect {
                palette,
                bits,
                data,
            } => palette[read_index(data, *bits, index)],
        }
    }

    pub fn set(&mut self, index: usize, value: T) -> T {
        match &mut self.storage {
            Storage::Single(current) => {
                let old = *current;
                if old == value {
                    return old;
                }
                let bits = S::MIN_BITS;
                let mut data = vec![0u64; packed_len(S::SIZE, bits)].into_boxed_slice();
                write_index(&mut data, bits, index, 1);
                self.storage = Storage::Indirect {
                    palette: vec![old, value],
                    bits,
                    data,
                };
                old
            }
            Storage::Indirect {
                palette,
                bits,
                data,
            } => {
                let old = palette[read_index(data, *bits, index)];
                if old == value {
                    return old;
                }
                let slot = match palette.iter().position(|p| *p == value) {
                    Some(slot) => slot,
                    None => {
                        if palette.len() == 1usize << *bits {
                            let grown = *bits + 1;
                            *data = repack(S::SIZE, data, *bits, grown);
                            *bits = grown;
                        }
                        palette.push(value);
                        palette.len() - 1
                    }
                };
                write_index(data, *bits, index, slot);
                old
            }
        }
    }

    pub fn fill(&mut self, value: T) {
        self.storage = Storage::Single(value);
    }

    pub fn single_value(&self) -> Option<T> {
        match &self.storage {
            Storage::Single(v) => Some(*v),
            Storage::Indirect { .. } => None,
        }
    }

    pub fn palette(&self) -> &[T] {
        match &self.storage {
            Storage::Single(v) => std::slice::from_ref(v),
            Storage::Indirect { palette, .. } => palette,
        }
    }

    pub fn bits(&self) -> u8 {
        match &self.storage {
            Storage::Single(_) => 0,
            Storage::Indirect { bits, .. } => *bits,
        }
    }

    pub fn packed(&self) -> &[u64] {
        match &self.storage {
            Storage::Single(_) => &[],
            Storage::Indirect { data, .. } => data,
        }
    }

    pub fn palette_index(&self, index: usize) -> usize {
        match &self.storage {
            Storage::Single(_) => 0,
            Storage::Indirect { bits, data, .. } => read_index(data, *bits, index),
        }
    }

    pub fn values(&self) -> impl Iterator<Item = T> + '_ {
        (0..S::SIZE).map(move |i| self.get(i))
    }

    pub fn compact(&mut self) {
        let Storage::Indirect {
            palette,
            bits,
            data,
        } = &self.storage
        else {
            return;
        };
        let mut used = vec![false; palette.len()];
        for i in 0..S::SIZE {
            used[read_index(data, *bits, i)] = true;
        }
        if used.iter().all(|u| *u) {
            return;
        }
        let mut remap = vec![0usize; palette.len()];
        let mut new_palette = Vec::with_capacity(palette.len());
        for (old, keep) in used.iter().enumerate() {
            if *keep {
                remap[old] = new_palette.len();
                new_palette.push(palette[old]);
            }
        }
        if new_palette.len() == 1 {
            self.storage = Storage::Single(new_palette[0]);
            return;
        }
        let new_bits = bits_for_palette(new_palette.len(), S::MIN_BITS);
        let mut new_data = vec![0u64; packed_len(S::SIZE, new_bits)].into_boxed_slice();
        for i in 0..S::SIZE {
            write_index(
                &mut new_data,
                new_bits,
                i,
                remap[read_index(data, *bits, i)],
            );
        }
        self.storage = Storage::Indirect {
            palette: new_palette,
            bits: new_bits,
            data: new_data,
        };
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;

    #[test]
    fn single_becomes_indirect_on_set() {
        let mut c = PalettedContainer::<u16, Blocks>::single(0);
        assert_eq!(c.set(5, 7), 0);
        assert_eq!(c.get(5), 7);
        assert_eq!(c.get(4), 0);
        assert_eq!(c.bits(), 4);
        assert_eq!(c.packed().len(), 256);
    }

    #[test]
    fn grows_bits_past_sixteen_entries() {
        let mut c = PalettedContainer::<u16, Blocks>::single(0);
        for i in 0..40u16 {
            c.set(usize::from(i), i + 1);
        }
        assert_eq!(c.bits(), 6);
        for i in 0..40u16 {
            assert_eq!(c.get(usize::from(i)), i + 1);
        }
        assert_eq!(c.get(4000), 0);
    }

    #[test]
    fn five_bit_packing_does_not_span_longs() {
        assert_eq!(packed_len(4096, 5), 342);
        assert_eq!(packed_len(64, 3), 4);
    }

    #[test]
    fn from_packed_rejects_bad_length() {
        let err =
            PalettedContainer::<u16, Blocks>::from_packed(vec![1, 2], 4, vec![0; 10]).unwrap_err();
        assert_eq!(
            err,
            PaletteError::DataLength {
                expected: 256,
                actual: 10
            }
        );
    }

    #[test]
    fn compact_drops_unused_entries() {
        let mut c = PalettedContainer::<u16, Biomes>::single(1);
        c.set(0, 2);
        c.set(0, 1);
        c.compact();
        assert_eq!(c.single_value(), Some(1));
    }

    proptest! {
        #[test]
        fn set_then_get_matches_model(ops in proptest::collection::vec((0usize..4096, 0u16..300), 0..500)) {
            let mut c = PalettedContainer::<u16, Blocks>::single(0);
            let mut model = vec![0u16; 4096];
            for (i, v) in ops {
                c.set(i, v);
                model[i] = v;
            }
            for (i, v) in model.iter().enumerate() {
                prop_assert_eq!(c.get(i), *v);
            }
            let rebuilt = PalettedContainer::<u16, Blocks>::from_packed(c.palette().to_vec(), c.bits().max(4), c.packed().to_vec());
            if c.single_value().is_none() {
                let rebuilt = rebuilt.unwrap();
                for (i, v) in model.iter().enumerate() {
                    prop_assert_eq!(rebuilt.get(i), *v);
                }
            }
        }
    }
}
