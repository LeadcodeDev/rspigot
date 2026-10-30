pub trait RandomSource {
    fn next_float(&mut self) -> f32;
    fn next_int(&mut self, bound: i32) -> i32;
}

const MULTIPLIER: i64 = 0x5_DEEC_E66D;
const INCREMENT: i64 = 0xB;
const MASK: i64 = (1 << 48) - 1;
const FLOAT_UNIT: f32 = 5.960_464_5e-8;
const DOUBLE_UNIT: f64 = 1.110_223_024_625_156_5e-16;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LegacyRandom {
    seed: i64,
}

impl LegacyRandom {
    pub fn new(seed: i64) -> Self {
        Self {
            seed: (seed ^ MULTIPLIER) & MASK,
        }
    }

    pub fn set_seed(&mut self, seed: i64) {
        self.seed = (seed ^ MULTIPLIER) & MASK;
    }

    pub fn decoration(seed: i64, min_block_x: i32, min_block_z: i32) -> Self {
        let mut random = Self::new(seed);
        let x_scale = random.next_long() | 1;
        let z_scale = random.next_long() | 1;
        let decoration = (i64::from(min_block_x).wrapping_mul(x_scale))
            .wrapping_add(i64::from(min_block_z).wrapping_mul(z_scale))
            ^ seed;
        random.set_seed(decoration);
        random
    }

    fn next(&mut self, bits: u32) -> i32 {
        self.seed = self.seed.wrapping_mul(MULTIPLIER).wrapping_add(INCREMENT) & MASK;
        (self.seed >> (48 - bits)) as i32
    }

    pub fn next_long(&mut self) -> i64 {
        let high = i64::from(self.next(32)) << 32;
        high.wrapping_add(i64::from(self.next(32)))
    }

    pub fn next_double(&mut self) -> f64 {
        let high = i64::from(self.next(26)) << 27;
        let low = i64::from(self.next(27));
        (high + low) as f64 * DOUBLE_UNIT
    }
}

impl RandomSource for LegacyRandom {
    fn next_float(&mut self) -> f32 {
        self.next(24) as f32 * FLOAT_UNIT
    }

    fn next_int(&mut self, bound: i32) -> i32 {
        if bound <= 0 {
            return 0;
        }
        if bound & (bound - 1) == 0 {
            return ((i64::from(bound) * i64::from(self.next(31))) >> 31) as i32;
        }
        loop {
            let bits = self.next(31);
            let value = bits % bound;
            if bits.wrapping_sub(value).wrapping_add(bound - 1) >= 0 {
                return value;
            }
        }
    }
}

pub fn random_between_inclusive(random: &mut impl RandomSource, min: i32, max: i32) -> i32 {
    random.next_int(max - min + 1) + min
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn legacy_random_matches_java_util_random() {
        let mut random = LegacyRandom::new(42);
        assert_eq!(random.next_int(10), 0);
        assert_eq!(random.next_int(10), 3);
        let mut random = LegacyRandom::new(42);
        assert_eq!(random.next_float(), 0.727_563_7);
        let mut random = LegacyRandom::new(42);
        assert_eq!(random.next_long(), -5_025_562_857_975_149_833);
        let mut random = LegacyRandom::new(42);
        assert_eq!(random.next_double(), 0.727_563_680_032_868_1);
    }

    #[test]
    fn decoration_seed_follows_worldgen_random() {
        let mut expected = LegacyRandom::new(1234);
        let x_scale = expected.next_long() | 1;
        let z_scale = expected.next_long() | 1;
        expected.set_seed(
            32i64
                .wrapping_mul(x_scale)
                .wrapping_add((-48i64).wrapping_mul(z_scale))
                ^ 1234,
        );
        let mut random = LegacyRandom::decoration(1234, 32, -48);
        assert_eq!(random.next_long(), expected.next_long());
        let mut random = LegacyRandom::new(5);
        assert!((0..100).all(|_| (3..=7).contains(&random_between_inclusive(&mut random, 3, 7))));
    }
}
