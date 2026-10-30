use super::EffectBehavior;

pub struct LastTick;

impl EffectBehavior for LastTick {
    fn should_tick(duration: i32, _amplifier: u8) -> bool {
        duration == 1
    }
}
