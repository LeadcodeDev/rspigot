use rspigot_nbt::Compound;

use super::base::EntityBase;
use super::data::SyncedValue;
use super::physics::{BlockView, Landing, MoveTraits, apply_inside_blocks, move_entity};
use crate::{BlockPos, BlockStateId, Vec3};

pub const DATA_START_POS: u8 = 8;
pub const FALLING_GRAVITY: f64 = 0.04;
pub const DEFAULT_FALL_HURT_MAX: i32 = 40;
const DRAG: f64 = 0.98;

#[derive(Debug, Clone, PartialEq)]
pub struct FallingState {
    pub state: BlockStateId,
    pub time: i32,
    pub drop_item: bool,
    pub hurt_entities: bool,
    pub fall_hurt_amount: f32,
    pub fall_hurt_max: i32,
    pub cancel_drop: bool,
    pub block_data: Option<Compound>,
}

impl FallingState {
    pub fn new(base: &mut EntityBase, state: BlockStateId, start: BlockPos) -> Self {
        base.data.define(
            DATA_START_POS,
            SyncedValue::BlockPos(BlockPos::new(0, 0, 0)),
        );
        base.data.set(DATA_START_POS, SyncedValue::BlockPos(start));
        Self {
            state,
            time: 0,
            drop_item: true,
            hurt_entities: false,
            fall_hurt_amount: 0.0,
            fall_hurt_max: DEFAULT_FALL_HURT_MAX,
            cancel_drop: false,
            block_data: None,
        }
    }
}

pub fn fall_step<V: BlockView>(
    base: &mut EntityBase,
    falling: &mut FallingState,
    view: &V,
) -> Option<Landing> {
    falling.time += 1;
    let gravity = if base.is_no_gravity() {
        0.0
    } else {
        FALLING_GRAVITY
    };
    if gravity != 0.0 {
        base.velocity = base.velocity + Vec3::new(0.0, -gravity, 0.0);
    }
    let traits = MoveTraits::plain(gravity);
    let landing = move_entity(base, view, base.velocity, &traits);
    apply_inside_blocks(base, view, false);
    landing
}

pub fn absorb_landing(base: &mut EntityBase) {
    base.velocity = base.velocity.multiply(0.7, -0.5, 0.7);
}

pub fn apply_drag(base: &mut EntityBase) {
    base.velocity = base.velocity.scale(DRAG);
}
