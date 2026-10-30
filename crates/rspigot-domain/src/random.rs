const MULTIPLIER: i64 = 0x5_DEEC_E66D;
const INCREMENT: i64 = 0xB;
const MASK: i64 = (1 << 48) - 1;
const FLOAT_UNIT: f32 = 5.960_464_5e-8;
const DOUBLE_UNIT: f64 = 1.110_223_024_625_156_5e-16;

pub trait RandomSource {
    fn next_float(&mut self) -> f32;

    fn next_int(&mut self, bound: i32) -> i32;

    fn next_double(&mut self) -> f64;

    fn next_boolean(&mut self) -> bool;

    fn next_gaussian(&mut self) -> f64;

    fn triangle(&mut self, mean: f64, spread: f64) -> f64 {
        mean + spread * (self.next_double() - self.next_double())
    }

    fn next_int_between(&mut self, min: i32, max_inclusive: i32) -> i32 {
        self.next_int(max_inclusive - min + 1) + min
    }
}

pub fn polar_gaussian(next_double: &mut impl FnMut() -> f64) -> (f64, f64) {
    loop {
        let v1 = 2.0 * next_double() - 1.0;
        let v2 = 2.0 * next_double() - 1.0;
        let s = v1 * v1 + v2 * v2;
        if s < 1.0 && s != 0.0 {
            let multiplier = (-2.0 * s.ln() / s).sqrt();
            return (v1 * multiplier, v2 * multiplier);
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LegacyRandom {
    seed: i64,
    next_gaussian: Option<f64>,
}

impl LegacyRandom {
    pub fn new(seed: i64) -> Self {
        Self {
            seed: (seed ^ MULTIPLIER) & MASK,
            next_gaussian: None,
        }
    }

    pub fn set_seed(&mut self, seed: i64) {
        self.seed = (seed ^ MULTIPLIER) & MASK;
        self.next_gaussian = None;
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
}

impl RandomSource for LegacyRandom {
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

    fn next_float(&mut self) -> f32 {
        self.next(24) as f32 * FLOAT_UNIT
    }

    fn next_double(&mut self) -> f64 {
        let high = i64::from(self.next(26)) << 27;
        let low = i64::from(self.next(27));
        (high + low) as f64 * DOUBLE_UNIT
    }

    fn next_boolean(&mut self) -> bool {
        self.next(1) != 0
    }

    fn next_gaussian(&mut self) -> f64 {
        if let Some(value) = self.next_gaussian.take() {
            return value;
        }
        let (first, second) = polar_gaussian(&mut || self.next_double());
        self.next_gaussian = Some(second);
        first
    }
}

pub fn random_between_inclusive(random: &mut impl RandomSource, min: i32, max: i32) -> i32 {
    random.next_int(max - min + 1) + min
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn matches_java_util_random() {
        let mut random = LegacyRandom::new(42);
        assert_eq!(random.next_int(10), 0);
        assert_eq!(random.next_int(10), 3);
        assert_eq!(random.next_int(10), 8);
        let mut random = LegacyRandom::new(42);
        assert_eq!(random.next_float(), 0.727_563_7);
        let mut random = LegacyRandom::new(42);
        assert_eq!(random.next_double(), 0.727_563_680_032_868_1);
        let mut random = LegacyRandom::new(0);
        assert_eq!(random.next_int(16), 11);
        let mut random = LegacyRandom::new(7);
        assert_eq!(
            [
                random.next_int(120),
                random.next_int(60),
                random.next_int(21)
            ],
            [76, 44, 18]
        );
    }

    #[test]
    fn gaussian_and_boolean_follow_java_util_random() {
        let mut random = LegacyRandom::new(42);
        assert!((random.next_gaussian() - 1.141_905_315_473_054_7).abs() < 1.0e-12);
        assert!((random.next_gaussian() - 0.919_407_948_982_787_9).abs() < 1.0e-12);
        let mut random = LegacyRandom::new(42);
        assert!(random.next_boolean());
        assert_eq!(
            LegacyRandom::new(42).next_long(),
            -5_025_562_857_975_149_833
        );
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
