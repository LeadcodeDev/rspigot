use crate::EffectType;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Effect {
    pub kind: EffectType,
    pub duration: i32,
    pub amplifier: u8,
    pub ambient: bool,
    pub particles: bool,
    pub icon: bool,
}

impl Effect {
    pub const INFINITE: i32 = -1;

    pub const fn new(kind: EffectType, duration: i32, amplifier: u8) -> Self {
        Self {
            kind,
            duration,
            amplifier,
            ambient: false,
            particles: true,
            icon: true,
        }
    }
}
