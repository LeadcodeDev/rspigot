use crate::{
    Block, BlockId, BlockSettings, FaceSupport, PerState, Property, ShapeId, StateFlags, StateId,
};

pub struct BlackStainedGlass;

impl Block for BlackStainedGlass {
    const NAME: &'static str = "minecraft:black_stained_glass";
    const ID: BlockId = BlockId(364);
    const FIRST_STATE: StateId = StateId(8783);
    const STATE_COUNT: u16 = 1;
    const DEFAULT_STATE: StateId = StateId(8783);
    const PROPERTIES: &'static [Property] = &[];
    const ITEM: Option<u16> = Some(620);
    const WALL_VARIANT: Option<BlockId> = None;
    const SETTINGS: BlockSettings = BlockSettings {
        hardness: 0.3,
        resistance: 0.3,
        friction: 0.6,
        speed_factor: 1.0,
        jump_factor: 1.0,
        requires_correct_tool: false,
        ignited_by_lava: false,
    };
    const LIGHT_EMISSION: PerState<u8> = PerState::Uniform(0);
    const LIGHT_DAMPENING: PerState<u8> = PerState::Uniform(0);
    const FLAGS: PerState<StateFlags> = PerState::Uniform(StateFlags(1062));
    const COLLISION: PerState<ShapeId> = PerState::Uniform(ShapeId(1));
    const FACE_SUPPORT: PerState<FaceSupport> = PerState::Uniform(FaceSupport(63, 63, 63));
    type Behavior = crate::family::Plain;
}
