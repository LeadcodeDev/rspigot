use super::EffectBehavior;

pub struct Interval<const TICKS: i32>;

impl<const TICKS: i32> EffectBehavior for Interval<TICKS> {
    fn should_tick(duration: i32, amplifier: u8) -> bool {
        let interval = TICKS.wrapping_shr(u32::from(amplifier));
        interval <= 0 || duration % interval == 0
    }
}
