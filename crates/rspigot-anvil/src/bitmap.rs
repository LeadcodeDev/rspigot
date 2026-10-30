#[derive(Debug, Default, Clone)]
pub struct SectorBitmap {
    words: Vec<u64>,
}

impl SectorBitmap {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn force(&mut self, start: u32, len: u32) {
        let end = start as usize + len as usize;
        self.ensure_capacity(end);
        for bit in start as usize..end {
            self.words[bit >> 6] |= 1u64 << (bit & 63);
        }
    }

    pub fn free(&mut self, start: u32, len: u32) {
        let end = (start as usize + len as usize).min(self.words.len() << 6);
        for bit in (start as usize).min(end)..end {
            self.words[bit >> 6] &= !(1u64 << (bit & 63));
        }
    }

    pub fn allocate(&mut self, len: u32) -> u32 {
        let len = len as usize;
        let mut start = 0usize;
        loop {
            start = self.next_clear(start);
            match self.next_set(start) {
                Some(next) if next - start < len => start = next,
                _ => break,
            }
        }
        self.ensure_capacity(start + len);
        for bit in start..start + len {
            self.words[bit >> 6] |= 1u64 << (bit & 63);
        }
        start as u32
    }

    #[cfg(test)]
    pub fn is_used(&self, sector: u32) -> bool {
        let bit = sector as usize;
        self.words
            .get(bit >> 6)
            .is_some_and(|word| word & (1u64 << (bit & 63)) != 0)
    }

    fn ensure_capacity(&mut self, bits: usize) {
        let needed = bits.div_ceil(64);
        if self.words.len() < needed {
            self.words.resize(needed, 0);
        }
    }

    fn next_clear(&self, from: usize) -> usize {
        let mut index = from >> 6;
        if index >= self.words.len() {
            return from;
        }
        let mut word = !self.words[index] & (u64::MAX << (from & 63));
        loop {
            if word != 0 {
                return (index << 6) + word.trailing_zeros() as usize;
            }
            index += 1;
            if index >= self.words.len() {
                return index << 6;
            }
            word = !self.words[index];
        }
    }

    fn next_set(&self, from: usize) -> Option<usize> {
        let mut index = from >> 6;
        if index >= self.words.len() {
            return None;
        }
        let mut word = self.words[index] & (u64::MAX << (from & 63));
        loop {
            if word != 0 {
                return Some((index << 6) + word.trailing_zeros() as usize);
            }
            index += 1;
            if index >= self.words.len() {
                return None;
            }
            word = self.words[index];
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn allocate_returns_first_fit_from_zero() {
        let mut bitmap = SectorBitmap::new();
        assert_eq!(bitmap.allocate(3), 0);
        assert_eq!(bitmap.allocate(2), 3);
        assert_eq!(bitmap.allocate(1), 5);
    }

    #[test]
    fn freed_sectors_are_reused() {
        let mut bitmap = SectorBitmap::new();
        bitmap.force(0, 2);
        let a = bitmap.allocate(3);
        let b = bitmap.allocate(3);
        let c = bitmap.allocate(3);
        assert_eq!((a, b, c), (2, 5, 8));
        bitmap.free(b, 3);
        assert_eq!(bitmap.allocate(2), 5);
        assert_eq!(bitmap.allocate(2), 11);
        assert_eq!(bitmap.allocate(1), 7);
    }

    #[test]
    fn gap_too_small_is_skipped() {
        let mut bitmap = SectorBitmap::new();
        bitmap.force(0, 2);
        bitmap.force(4, 2);
        assert_eq!(bitmap.allocate(3), 6);
        assert_eq!(bitmap.allocate(2), 2);
    }

    #[test]
    fn force_and_free_cross_word_boundaries() {
        let mut bitmap = SectorBitmap::new();
        bitmap.force(60, 10);
        assert!(bitmap.is_used(63));
        assert!(bitmap.is_used(64));
        assert!(bitmap.is_used(69));
        assert!(!bitmap.is_used(70));
        assert!(!bitmap.is_used(59));
        bitmap.free(62, 5);
        assert!(bitmap.is_used(61));
        assert!(!bitmap.is_used(62));
        assert!(!bitmap.is_used(66));
        assert!(bitmap.is_used(67));
    }

    #[test]
    fn allocate_spanning_many_words() {
        let mut bitmap = SectorBitmap::new();
        bitmap.force(0, 2);
        assert_eq!(bitmap.allocate(255), 2);
        assert_eq!(bitmap.allocate(1), 257);
    }

    #[test]
    fn free_beyond_capacity_is_a_no_op() {
        let mut bitmap = SectorBitmap::new();
        bitmap.free(1000, 10);
        assert!(!bitmap.is_used(1005));
        assert_eq!(bitmap.allocate(1), 0);
    }
}
