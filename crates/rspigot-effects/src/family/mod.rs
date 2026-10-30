mod every_tick;
mod instantaneous;
mod interval;
mod last_tick;

pub use every_tick::EveryTick;
pub use instantaneous::Instantaneous;
pub use interval::Interval;
pub use last_tick::LastTick;

pub trait EffectBehavior {
    fn should_tick(_duration: i32, _amplifier: u8) -> bool {
        false
    }
}

pub struct Plain;

impl EffectBehavior for Plain {}
