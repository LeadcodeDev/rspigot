use rspigot_blocks::StateFlags;

use super::base::EntityBase;
use super::data::SyncedValue;
use super::item::move_towards_closest_space;
use super::physics::{
    BlockView, MoveTraits, apply_inside_blocks, block_colliders, friction, move_entity, on_pos,
    water_height,
};
use crate::Vec3;

pub const DATA_VALUE: u8 = 8;
pub const LIFETIME: i32 = 6000;
pub const SCAN_PERIOD: i32 = 20;
pub const MAX_FOLLOW_DISTANCE: f64 = 8.0;
pub const ORB_GROUPS_PER_AREA: i32 = 40;
pub const MERGE_INFLATE: f64 = 0.5;
pub const AWARD_MERGE_SIZE: f64 = 1.0;
pub const DEFAULT_HEALTH: i32 = 5;
pub const GRAVITY: f64 = 0.03;
const AIR_DRAG: f32 = 0.98;
const BELOW_OFFSET: f32 = 0.999_999;
const SPAWN_SPEED: f64 = 0.2;
const SPAWN_SPREAD: f64 = 0.1;
const SPAWN_SCALE: f64 = 2.0;
const LAVA_SPREAD: f32 = 0.2;
const LAVA_LIFT: f64 = 0.200_000_002_980_232_24;
const WATER_DRAG: f64 = 0.990_000_009_536_743_2;
const WATER_RISE: f64 = 5.000_000_237_487_257e-4;
const WATER_RISE_CAP: f64 = 0.059_999_998_658_895_49;
const BOUNCE: f64 = 0.4;
const FOLLOW_RESET_SQR: f64 = 64.0;
const FOLLOW_PULL: f64 = 0.1;
const EYE_FLUID_OFFSET: f64 = 0.111_111_111_938_953_4;
const STUCK_DEFLATE: f64 = 1.0e-7;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct OrbState {
    pub age: i32,
    pub health: i32,
    pub count: i32,
    pub following: Option<crate::player::PlayerId>,
}

impl OrbState {
    pub fn new(base: &mut EntityBase, value: i32) -> Self {
        base.data.define(DATA_VALUE, SyncedValue::Int(0));
        set_value(base, value);
        Self {
            age: 0,
            health: DEFAULT_HEALTH,
            count: 1,
            following: None,
        }
    }
}

pub fn value(base: &EntityBase) -> i32 {
    base.data.int(DATA_VALUE)
}

pub fn set_value(base: &mut EntityBase, value: i32) {
    base.data.set(DATA_VALUE, SyncedValue::Int(value));
}

pub fn experience_value(value: i32) -> i32 {
    const LARGE: [i32; 15] = [
        81_335_063, 40_667_527, 20_333_759, 10_166_857, 5_083_423, 2_541_701, 1_270_849, 635_413,
        317_701, 158_849, 79_423, 39_709, 19_853, 9_923, 4_957,
    ];
    const SMALL: [i32; 10] = [2477, 1237, 617, 307, 149, 73, 37, 17, 7, 3];
    if value > 162_670_129 {
        return value - 100_000;
    }
    if let Some(step) = LARGE.iter().find(|step| value > **step) {
        return *step;
    }
    SMALL
        .iter()
        .find(|step| value >= **step)
        .copied()
        .unwrap_or(1)
}

pub fn spawn_velocity(next_double: &mut impl FnMut() -> f64) -> Vec3 {
    let x = (next_double() * SPAWN_SPEED - SPAWN_SPREAD) * SPAWN_SCALE;
    let y = next_double() * SPAWN_SPEED * SPAWN_SCALE;
    let z = (next_double() * SPAWN_SPEED - SPAWN_SPREAD) * SPAWN_SCALE;
    Vec3::new(x, y, z)
}

pub fn can_merge(id: i32, value: i32, group: i32, wanted_value: i32) -> bool {
    (id - group) % ORB_GROUPS_PER_AREA == 0 && value == wanted_value
}

fn eye_in_water(base: &EntityBase, view: &impl BlockView) -> bool {
    let eye = base.position.y + f64::from(base.kind.dimensions().eye_height) - EYE_FLUID_OFFSET;
    let pos = crate::Vec3::new(base.position.x, eye, base.position.z).block();
    water_height(view, pos).is_some_and(|height| eye < f64::from(pos.y) + height)
}

fn in_lava(base: &EntityBase, view: &impl BlockView) -> bool {
    view.block_or_air(base.position.block())
        .flags()
        .contains(StateFlags::LAVA)
}

fn stuck(base: &EntityBase, view: &impl BlockView, offset: Vec3) -> bool {
    let bb = base.bounding_box().move_by(offset).deflate(STUCK_DEFLATE);
    !block_colliders(view, &bb).is_empty()
}

pub fn pull_towards(base: &mut EntityBase, target: Vec3, eye_height: f64) {
    let delta = Vec3::new(
        target.x - base.position.x,
        target.y + eye_height / 2.0 - base.position.y,
        target.z - base.position.z,
    );
    let length = delta.length_sqr().sqrt();
    let strength = 1.0 - length / MAX_FOLLOW_DISTANCE;
    base.velocity = base.velocity + delta.normalize().scale(strength * strength * FOLLOW_PULL);
}

pub fn keeps_following(distance_sqr: f64) -> bool {
    distance_sqr <= FOLLOW_RESET_SQR
}

pub fn before_move<V: BlockView>(
    base: &mut EntityBase,
    view: &V,
    next_float: &mut impl FnMut() -> f32,
) -> bool {
    let stuck_now = stuck(base, view, Vec3::ZERO);
    if eye_in_water(base, view) {
        let v = base.velocity;
        base.velocity = Vec3::new(
            v.x * WATER_DRAG,
            (v.y + WATER_RISE).min(WATER_RISE_CAP),
            v.z * WATER_DRAG,
        );
    } else if !stuck_now && !base.is_no_gravity() {
        base.velocity = base.velocity + Vec3::new(0.0, -GRAVITY, 0.0);
    }
    if in_lava(base, view) {
        let x = (next_float() - next_float()) * LAVA_SPREAD;
        let z = (next_float() - next_float()) * LAVA_SPREAD;
        base.velocity = Vec3::new(f64::from(x), LAVA_LIFT, f64::from(z));
    }
    stuck_now
}

pub fn after_follow<V: BlockView>(
    base: &mut EntityBase,
    orb: &mut OrbState,
    view: &V,
    following: bool,
    stuck_now: bool,
    next_float: &mut impl FnMut() -> f32,
) -> bool {
    if !following && stuck_now && stuck(base, view, base.velocity) {
        let bb = base.bounding_box();
        let target = Vec3::new(
            base.position.x,
            (bb.min_y + bb.max_y) / 2.0,
            base.position.z,
        );
        move_towards_closest_space(base, view, target, next_float);
        base.needs_sync = true;
    }
    let gravity = if base.is_no_gravity() { 0.0 } else { GRAVITY };
    let previous = base.velocity.y;
    let traits = MoveTraits::plain(gravity);
    move_entity(base, view, base.velocity, &traits);
    apply_inside_blocks(base, view, false);
    let mut drag = AIR_DRAG;
    if base.on_ground {
        drag *= friction(view.block_or_air(on_pos(base, view, BELOW_OFFSET)));
    }
    base.velocity = base.velocity.scale(f64::from(drag));
    if base.vertical_collision_below && previous < -gravity {
        base.velocity = Vec3::new(base.velocity.x, -previous * BOUNCE, base.velocity.z);
    }
    orb.age += 1;
    orb.age >= LIFETIME
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn values_split_into_the_vanilla_ladder() {
        let split = |mut total: i32| {
            let mut out = Vec::new();
            while total > 0 {
                let value = experience_value(total);
                total -= value;
                out.push(value);
            }
            out
        };
        assert_eq!(split(1), vec![1]);
        assert_eq!(split(3), vec![3]);
        assert_eq!(split(5), vec![3, 1, 1]);
        assert_eq!(split(10), vec![7, 3]);
        assert_eq!(split(21), vec![17, 3, 1]);
        assert_eq!(split(100), vec![73, 17, 7, 3]);
        assert_eq!(experience_value(4957), 2477);
        assert_eq!(experience_value(4958), 4957);
        assert_eq!(experience_value(200_000_000), 199_900_000);
    }

    #[test]
    fn merge_groups_are_forty_ids_wide() {
        assert!(can_merge(45, 3, 5, 3));
        assert!(can_merge(5, 3, 45, 3));
        assert!(!can_merge(46, 3, 5, 3));
        assert!(!can_merge(45, 7, 5, 3));
    }

    #[test]
    fn spawn_velocity_spreads_up_and_sideways() {
        let mut values = [1.0, 0.5, 0.0].into_iter();
        let mut next = || values.next().unwrap_or(0.0);
        let v = spawn_velocity(&mut next);
        assert!((v.x - 0.2).abs() < 1e-12);
        assert!((v.y - 0.2).abs() < 1e-12);
        assert!((v.z + 0.2).abs() < 1e-12);
    }
}
