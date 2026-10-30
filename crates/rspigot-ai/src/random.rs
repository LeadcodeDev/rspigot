const MULTIPLIER: i64 = 0x5_DEEC_E66D;
const INCREMENT: i64 = 0xB;
const MASK: i64 = (1 << 48) - 1;
const FLOAT_UNIT: f32 = 5.960_464_5e-8;
const DOUBLE_UNIT: f64 = 1.110_223_024_625_156_5e-16;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MobRandom {
    seed: i64,
}

impl MobRandom {
    pub fn new(seed: i64) -> Self {
        Self {
            seed: (seed ^ MULTIPLIER) & MASK,
        }
    }

    fn next(&mut self, bits: u32) -> i32 {
        self.seed = self.seed.wrapping_mul(MULTIPLIER).wrapping_add(INCREMENT) & MASK;
        (self.seed >> (48 - bits)) as i32
    }

    pub fn next_int(&mut self, bound: i32) -> i32 {
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

    pub fn next_float(&mut self) -> f32 {
        self.next(24) as f32 * FLOAT_UNIT
    }

    pub fn next_double(&mut self) -> f64 {
        let high = i64::from(self.next(26)) << 27;
        let low = i64::from(self.next(27));
        (high + low) as f64 * DOUBLE_UNIT
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn matches_java_util_random() {
        let mut random = MobRandom::new(42);
        assert_eq!(random.next_int(10), 0);
        assert_eq!(random.next_int(10), 3);
        assert_eq!(random.next_int(10), 8);
        let mut random = MobRandom::new(42);
        assert_eq!(random.next_float(), 0.727_563_7);
        let mut random = MobRandom::new(42);
        assert_eq!(random.next_double(), 0.727_563_680_032_868_1);
        let mut random = MobRandom::new(0);
        assert_eq!(random.next_int(16), 11);
        let mut random = MobRandom::new(7);
        assert_eq!(
            [
                random.next_int(120),
                random.next_int(60),
                random.next_int(21)
            ],
            [76, 44, 18]
        );
    }
}
