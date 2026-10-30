use crate::{
    Block, BlockId, BlockSettings, FaceSupport, PerState, Property, ShapeId, StateFlags, StateId,
};

pub struct Cobweb;

impl Block for Cobweb {
    const NAME: &'static str = "minecraft:cobweb";
    const ID: BlockId = BlockId(139);
    const FIRST_STATE: StateId = StateId(2362);
    const STATE_COUNT: u16 = 1;
    const DEFAULT_STATE: StateId = StateId(2362);
    const PROPERTIES: &'static [Property] = &[];
    const ITEM: Option<u16> = Some(237);
    const WALL_VARIANT: Option<BlockId> = None;
    const SETTINGS: BlockSettings = BlockSettings {
        hardness: 4.0,
        resistance: 4.0,
        friction: 0.6,
        speed_factor: 1.0,
        jump_factor: 1.0,
        bounce_restitution: 0.0,
        fall_distance_reduction: 0.0,
        requires_correct_tool: true,
        ignited_by_lava: false,
    };
    const LIGHT_EMISSION: PerState<u8> = PerState::Uniform(0);
    const LIGHT_DAMPENING: PerState<u8> = PerState::Uniform(1);
    const FLAGS: PerState<StateFlags> = PerState::Uniform(StateFlags(2048));
    const COLLISION: PerState<ShapeId> = PerState::Uniform(ShapeId(0));
    const FACE_SUPPORT: PerState<FaceSupport> = PerState::Uniform(FaceSupport(0, 0, 0));
    type Behavior = crate::family::Plain;
}
