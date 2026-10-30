use crate::world::Difficulty;

const BASE_SCALE: f32 = 0.75;
const GLOBAL_OFFSET: f32 = -72_000.0;
const GLOBAL_SPAN: f32 = 1_440_000.0;
const GLOBAL_WEIGHT: f32 = 0.25;
const LOCAL_SPAN: f32 = 3_600_000.0;
const LOCAL_WEIGHT_NORMAL: f32 = 0.75;
const MOON_WEIGHT: f32 = 0.25;
const EASY_FACTOR: f32 = 0.5;
const DAY_TICKS: i64 = 24_000;
const MOON_BRIGHTNESS: [f32; 8] = [1.0, 0.75, 0.5, 0.25, 0.0, 0.25, 0.5, 0.75];
const MONSTERS_STOP_BURNING: i64 = 12_542;
const MONSTERS_START_BURNING: i64 = 23_460;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct DifficultyInstance {
    pub base: Difficulty,
    pub effective: f32,
}

fn clamp(value: f32, min: f32, max: f32) -> f32 {
    if value < min {
        min
    } else if value > max {
        max
    } else {
        value
    }
}

impl DifficultyInstance {
    pub fn new(base: Difficulty, total_time: i64, local_time: i64, moon_brightness: f32) -> Self {
        Self {
            base,
            effective: calculate(base, total_time, local_time, moon_brightness),
        }
    }

    pub fn special_multiplier(&self) -> f32 {
        if self.effective < 2.0 {
            0.0
        } else if self.effective > 4.0 {
            1.0
        } else {
            (self.effective - 2.0) / 2.0
        }
    }

    pub fn is_harder_than(&self, value: f32) -> bool {
        self.effective > value
    }
}

fn calculate(base: Difficulty, total_time: i64, local_time: i64, moon_brightness: f32) -> f32 {
    if base == Difficulty::Peaceful {
        return 0.0;
    }
    let hard = base == Difficulty::Hard;
    let mut scale = BASE_SCALE;
    let global = clamp((total_time as f32 + GLOBAL_OFFSET) / GLOBAL_SPAN, 0.0, 1.0) * GLOBAL_WEIGHT;
    scale += global;
    let mut local = 0.0f32;
    local += clamp(local_time as f32 / LOCAL_SPAN, 0.0, 1.0)
        * if hard { 1.0 } else { LOCAL_WEIGHT_NORMAL };
    local += clamp(moon_brightness * MOON_WEIGHT, 0.0, global);
    if base == Difficulty::Easy {
        local *= EASY_FACTOR;
    }
    scale += local;
    f32::from(base.id()) * scale
}

pub fn moon_brightness(day_time: i64, has_day_cycle: bool) -> f32 {
    if !has_day_cycle {
        return MOON_BRIGHTNESS[0];
    }
    MOON_BRIGHTNESS[(day_time.div_euclid(DAY_TICKS).rem_euclid(8)) as usize]
}

pub fn monsters_burn(day_time: i64, has_day_cycle: bool) -> bool {
    if !has_day_cycle {
        return false;
    }
    let time = day_time.rem_euclid(DAY_TICKS);
    !(MONSTERS_STOP_BURNING..MONSTERS_START_BURNING).contains(&time)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_fresh_world_scales_the_base_difficulty() {
        let easy = DifficultyInstance::new(Difficulty::Easy, 0, 0, 1.0);
        assert_eq!(easy.effective, 0.75);
        let hard = DifficultyInstance::new(Difficulty::Hard, 0, 0, 1.0);
        assert_eq!(hard.effective, 2.25);
        assert_eq!(hard.special_multiplier(), 0.125);
        assert_eq!(
            DifficultyInstance::new(Difficulty::Peaceful, 10, 10, 1.0).effective,
            0.0
        );
    }

    #[test]
    fn old_inhabited_chunks_under_a_full_moon_reach_full_difficulty() {
        let hard = DifficultyInstance::new(Difficulty::Hard, 2_000_000, 4_000_000, 1.0);
        assert_eq!(hard.effective, 3.0 * (0.75 + 0.25 + 1.0 + 0.25));
        assert_eq!(hard.special_multiplier(), 1.0);
    }

    #[test]
    fn day_timeline_controls_moon_and_burning() {
        assert!(monsters_burn(1000, true));
        assert!(!monsters_burn(13_000, true));
        assert!(monsters_burn(23_460, true));
        assert!(!monsters_burn(1000, false));
        assert_eq!(moon_brightness(0, true), 1.0);
        assert_eq!(moon_brightness(24_000 * 4 + 5, true), 0.0);
    }
}
