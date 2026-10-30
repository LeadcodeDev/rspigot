use crate::{
    Block, BlockId, BlockSettings, FaceSupport, PerState, Property, ShapeId, StateFlags, StateId,
};

pub struct BrownConcretePowder;

impl Block for BrownConcretePowder {
    const NAME: &'static str = "minecraft:brown_concrete_powder";
    const ID: BlockId = BlockId(828);
    const FIRST_STATE: StateId = StateId(18415);
    const STATE_COUNT: u16 = 1;
    const DEFAULT_STATE: StateId = StateId(18415);
    const PROPERTIES: &'static [Property] = &[];
    const ITEM: Option<u16> = Some(749);
    const WALL_VARIANT: Option<BlockId> = None;
    const SETTINGS: BlockSettings = BlockSettings {
        hardness: 0.5,
        resistance: 0.5,
        friction: 0.6,
        speed_factor: 1.0,
        jump_factor: 1.0,
        requires_correct_tool: false,
        ignited_by_lava: false,
    };
    const LIGHT_EMISSION: PerState<u8> = PerState::Uniform(0);
    const LIGHT_DAMPENING: PerState<u8> = PerState::Uniform(15);
    const FLAGS: PerState<StateFlags> = PerState::Uniform(StateFlags(1054));
    const COLLISION: PerState<ShapeId> = PerState::Uniform(ShapeId(1));
    const FACE_SUPPORT: PerState<FaceSupport> = PerState::Uniform(FaceSupport(63, 63, 63));
    type Behavior = crate::family::ConcretePowder;
}
