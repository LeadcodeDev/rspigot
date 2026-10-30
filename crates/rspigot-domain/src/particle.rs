use crate::{ItemId, Vec3};

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Particle {
    pub kind: &'static str,
    pub item: Option<ItemId>,
    pub position: Vec3,
    pub count: i32,
    pub spread: Vec3,
    pub speed: f64,
}

impl Particle {
    pub const fn single(kind: &'static str, position: Vec3) -> Self {
        Self {
            kind,
            item: None,
            position,
            count: 1,
            spread: Vec3::ZERO,
            speed: 0.0,
        }
    }
}
