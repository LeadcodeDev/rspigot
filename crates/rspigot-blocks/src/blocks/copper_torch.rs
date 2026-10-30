use crate::{
    Block, BlockId, BlockSettings, FaceSupport, PerState, Property, ShapeId, StateFlags, StateId,
};

pub struct CopperTorch;

impl Block for CopperTorch {
    const NAME: &'static str = "minecraft:copper_torch";
    const ID: BlockId = BlockId(341);
    const FIRST_STATE: StateId = StateId(8681);
    const STATE_COUNT: u16 = 1;
    const DEFAULT_STATE: StateId = StateId(8681);
    const PROPERTIES: &'static [Property] = &[];
    const ITEM: Option<u16> = Some(440);
    const WALL_VARIANT: Option<BlockId> = Some(BlockId(342));
    const SETTINGS: BlockSettings = BlockSettings {
        hardness: 0.0,
        resistance: 0.0,
        friction: 0.6,
        speed_factor: 1.0,
        jump_factor: 1.0,
        bounce_restitution: 0.0,
        fall_distance_reduction: 0.0,
        requires_correct_tool: false,
        ignited_by_lava: false,
    };
    const LIGHT_EMISSION: PerState<u8> = PerState::Uniform(14);
    const LIGHT_DAMPENING: PerState<u8> = PerState::Uniform(0);
    const FLAGS: PerState<StateFlags> = PerState::Uniform(StateFlags(32));
    const COLLISION: PerState<ShapeId> = PerState::Uniform(ShapeId(0));
    const FACE_SUPPORT: PerState<FaceSupport> = PerState::Uniform(FaceSupport(0, 0, 0));
    type Behavior = crate::family::StandingWithWall;
}
