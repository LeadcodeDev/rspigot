use super::EffectBehavior;

pub struct EveryTick;

impl EffectBehavior for EveryTick {
    fn should_tick(_duration: i32, _amplifier: u8) -> bool {
        true
    }
}
