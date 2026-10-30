use crate::{
    Block, BlockId, BlockSettings, FaceSupport, PerState, Property, ShapeId, StateFlags, StateId,
};

pub struct PottedPoppy;

impl Block for PottedPoppy {
    const NAME: &'static str = "minecraft:potted_poppy";
    const ID: BlockId = BlockId(477);
    const FIRST_STATE: StateId = StateId(12386);
    const STATE_COUNT: u16 = 1;
    const DEFAULT_STATE: StateId = StateId(12386);
    const PROPERTIES: &'static [Property] = &[];
    const ITEM: Option<u16> = None;
    const WALL_VARIANT: Option<BlockId> = None;
    const SETTINGS: BlockSettings = BlockSettings {
        hardness: 0.0,
        resistance: 0.0,
        friction: 0.6,
        speed_factor: 1.0,
        jump_factor: 1.0,
        requires_correct_tool: false,
        ignited_by_lava: false,
    };
    const LIGHT_EMISSION: PerState<u8> = PerState::Uniform(0);
    const LIGHT_DAMPENING: PerState<u8> = PerState::Uniform(0);
    const FLAGS: PerState<StateFlags> = PerState::Uniform(StateFlags(36));
    const COLLISION: PerState<ShapeId> = PerState::Uniform(ShapeId(177));
    const FACE_SUPPORT: PerState<FaceSupport> = PerState::Uniform(FaceSupport(0, 1, 0));
    type Behavior = crate::family::Plain;
}
