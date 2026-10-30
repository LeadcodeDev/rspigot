use super::EffectBehavior;

pub struct Instantaneous;

impl EffectBehavior for Instantaneous {
    fn should_tick(duration: i32, _amplifier: u8) -> bool {
        duration >= 1
    }
}
