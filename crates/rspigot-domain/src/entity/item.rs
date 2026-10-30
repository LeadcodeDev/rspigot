use rspigot_blocks::StateFlags;
use uuid::Uuid;

use super::aabb::Axis;
use super::base::EntityBase;
use super::data::SyncedValue;
use super::physics::{
    BlockView, Landing, MoveTraits, apply_inside_blocks, block_colliders, friction, move_entity,
    movement_affecting_pos,
};
use crate::item::ItemStack;
use crate::{BlockPos, Vec3};

pub const DATA_ITEM: u8 = 8;
pub const ITEM_GRAVITY: f64 = 0.04;
pub const INFINITE_PICKUP_DELAY: i32 = 32767;
pub const INFINITE_LIFETIME: i32 = -32768;
pub const DEFAULT_HEALTH: i32 = 5;
pub const THROWN_PICKUP_DELAY: i32 = 40;
pub const THROWN_DROP_BELOW_EYES: f32 = 0.3;
const DEGREES_TO_RADIANS: f32 = 0.017_453_292;
const THROW_SPEED: f32 = 0.3;
const THROW_LIFT: f32 = 0.1;
const THROW_SPREAD: f32 = 0.02;
const FULL_TURN: f32 = 6.283_185_5;
const SIN_SCALE: f64 = 10_430.378_350_470_453;
const SIN_QUARTER: f64 = 16_384.0;
const AIR_DRAG: f32 = 0.98;
const BOUNCE: f64 = -0.5;
const SYNC_DELTA: f64 = 0.01;
const RESTING_SPEED: f64 = 9.999_999_747_378_752e-6;
const NO_PHYSICS_DEFLATE: f64 = 1.0e-7;

#[derive(Debug, Clone, PartialEq)]
pub struct ItemState {
    pub age: i32,
    pub pickup_delay: i32,
    pub health: i32,
    pub thrower: Option<Uuid>,
    pub owner: Option<Uuid>,
}

impl ItemState {
    pub fn new(base: &mut EntityBase, stack: ItemStack, pickup_delay: i32) -> Self {
        base.data.define(DATA_ITEM, SyncedValue::Item(None));
        base.data.set(DATA_ITEM, SyncedValue::Item(Some(stack)));
        Self {
            age: 0,
            pickup_delay,
            health: DEFAULT_HEALTH,
            thrower: None,
            owner: None,
        }
    }
}

fn sin_table(index: i64) -> f32 {
    let index = (index & 65_535) as f64;
    (index * std::f64::consts::PI * 2.0 / 65_536.0).sin() as f32
}

pub fn mth_sin(value: f64) -> f32 {
    sin_table((value * SIN_SCALE) as i64)
}

pub fn mth_cos(value: f64) -> f32 {
    sin_table((value * SIN_SCALE + SIN_QUARTER) as i64)
}

pub fn wrap_degrees(value: f32) -> f32 {
    let mut wrapped = value % 360.0;
    if wrapped >= 180.0 {
        wrapped -= 360.0;
    }
    if wrapped < -180.0 {
        wrapped += 360.0;
    }
    wrapped
}

pub fn thrown_velocity(yaw: f32, pitch: f32, rolls: [f32; 4]) -> Vec3 {
    let yaw = wrap_degrees(yaw);
    let sin_pitch = mth_sin(f64::from(pitch * DEGREES_TO_RADIANS));
    let cos_pitch = mth_cos(f64::from(pitch * DEGREES_TO_RADIANS));
    let sin_yaw = mth_sin(f64::from(yaw * DEGREES_TO_RADIANS));
    let cos_yaw = mth_cos(f64::from(yaw * DEGREES_TO_RADIANS));
    let angle = f64::from(rolls[0] * FULL_TURN);
    let spread = f64::from(THROW_SPREAD * rolls[1]);
    Vec3::new(
        f64::from(-sin_yaw * cos_pitch * THROW_SPEED) + angle.cos() * spread,
        f64::from(-sin_pitch * THROW_SPEED + THROW_LIFT + (rolls[2] - rolls[3]) * THROW_LIFT),
        f64::from(cos_yaw * cos_pitch * THROW_SPEED) + angle.sin() * spread,
    )
}

pub fn stack(base: &EntityBase) -> Option<&ItemStack> {
    match base.data.get(DATA_ITEM) {
        Some(SyncedValue::Item(stack)) => stack.as_ref(),
        _ => None,
    }
}

pub fn set_stack(base: &mut EntityBase, stack: Option<ItemStack>) {
    base.data.set(DATA_ITEM, SyncedValue::Item(stack));
}

pub fn move_towards_closest_space<V: BlockView>(
    base: &mut EntityBase,
    view: &V,
    target: Vec3,
    next_float: &mut impl FnMut() -> f32,
) {
    let pos = target.block();
    let delta = Vec3::new(
        target.x - f64::from(pos.x),
        target.y - f64::from(pos.y),
        target.z - f64::from(pos.z),
    );
    let directions: [(Axis, i32, (i32, i32, i32)); 5] = [
        (Axis::Z, -1, (0, 0, -1)),
        (Axis::Z, 1, (0, 0, 1)),
        (Axis::X, -1, (-1, 0, 0)),
        (Axis::X, 1, (1, 0, 0)),
        (Axis::Y, 1, (0, 1, 0)),
    ];
    let mut best = (Axis::Y, 1);
    let mut closest = f64::MAX;
    for (axis, step, (dx, dy, dz)) in directions {
        let neighbour = BlockPos::new(pos.x + dx, pos.y + dy, pos.z + dz);
        let full = view
            .block_or_air(neighbour)
            .flags()
            .contains(StateFlags::FULL_COLLISION_CUBE);
        if !full {
            let coord = axis.of(delta);
            let distance = if step > 0 { 1.0 - coord } else { coord };
            if distance < closest {
                closest = distance;
                best = (axis, step);
            }
        }
    }
    let speed = next_float() * 0.2 + 0.1;
    let push = f64::from(best.1 as f32 * speed);
    let scaled = base.velocity.scale(0.75);
    base.velocity = best.0.with(scaled, push);
}

#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct ItemTick {
    pub landing: Option<Landing>,
    pub moved: bool,
}

pub fn tick_item<V: BlockView>(
    base: &mut EntityBase,
    item: &mut ItemState,
    view: &V,
    next_float: &mut impl FnMut() -> f32,
) -> ItemTick {
    if item.pickup_delay != INFINITE_PICKUP_DELAY {
        item.pickup_delay -= 1;
    }
    if item.age != INFINITE_LIFETIME {
        item.age += 1;
    }
    let before = base.velocity;
    let gravity = if base.is_no_gravity() {
        0.0
    } else {
        ITEM_GRAVITY
    };
    if gravity != 0.0 {
        base.velocity = base.velocity + Vec3::new(0.0, -gravity, 0.0);
    }
    let inside = block_colliders(view, &base.bounding_box().deflate(NO_PHYSICS_DEFLATE));
    base.no_physics = !inside.is_empty();
    if base.no_physics {
        let bb = base.bounding_box();
        let target = Vec3::new(
            base.position.x,
            (bb.min_y + bb.max_y) / 2.0,
            base.position.z,
        );
        move_towards_closest_space(base, view, target, next_float);
    }
    let mut outcome = ItemTick::default();
    if !base.on_ground
        || base.velocity.horizontal_distance_sqr() > RESTING_SPEED
        || (base.tick_count + base.id.0) % 4 == 0
    {
        let traits = MoveTraits::plain(gravity);
        outcome.landing = move_entity(base, view, base.velocity, &traits);
        outcome.moved = true;
        apply_inside_blocks(base, view, false);
        let mut horizontal = AIR_DRAG;
        if base.on_ground {
            horizontal = AIR_DRAG * friction(view.block_or_air(movement_affecting_pos(base, view)));
        }
        base.velocity = base.velocity.multiply(
            f64::from(horizontal),
            f64::from(AIR_DRAG),
            f64::from(horizontal),
        );
        if base.on_ground && base.velocity.y < 0.0 {
            base.velocity = base.velocity.multiply(1.0, BOUNCE, 1.0);
        }
    }
    if (base.velocity - before).length_sqr() > SYNC_DELTA {
        base.needs_sync = true;
    }
    outcome
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sine_table_matches_vanilla_mth_for_both_signs() {
        assert_eq!(mth_sin(0.0), 0.0);
        assert_eq!(mth_sin(std::f64::consts::FRAC_PI_2), 1.0);
        assert_eq!(mth_sin(-std::f64::consts::FRAC_PI_2), -1.0);
        assert_eq!(mth_cos(0.0), 1.0);
        assert_eq!(wrap_degrees(270.0), -90.0);
        assert_eq!(wrap_degrees(-190.0), 170.0);
    }

    #[test]
    fn thrown_velocity_follows_the_look_direction() {
        let ahead = thrown_velocity(0.0, 0.0, [0.0, 0.0, 0.5, 0.5]);
        assert_eq!(ahead, Vec3::new(0.0, f64::from(0.1f32), f64::from(0.3f32)));
        let east = thrown_velocity(270.0, 0.0, [0.0, 0.0, 0.5, 0.5]);
        assert!(
            (east.x - 0.3).abs() < 1e-6 && east.z.abs() < 1e-6,
            "{east:?}"
        );
        let up = thrown_velocity(0.0, -90.0, [0.0, 0.0, 1.0, 0.0]);
        assert!((up.y - 0.5).abs() < 1e-6 && up.z.abs() < 1e-6, "{up:?}");
        let spread = thrown_velocity(0.0, 0.0, [0.25, 1.0, 0.5, 0.5]);
        assert!(spread.x.abs() < 1e-6, "{spread:?}");
        assert!((spread.z - 0.32).abs() < 1e-6, "{spread:?}");
    }
}
