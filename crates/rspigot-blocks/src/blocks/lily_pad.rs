use crate::{
    Block, BlockId, BlockSettings, FaceSupport, PerState, Property, ShapeId, StateFlags, StateId,
};

pub struct LilyPad;

impl Block for LilyPad {
    const NAME: &'static str = "minecraft:lily_pad";
    const ID: BlockId = BlockId(424);
    const FIRST_STATE: StateId = StateId(10654);
    const STATE_COUNT: u16 = 1;
    const DEFAULT_STATE: StateId = StateId(10654);
    const PROPERTIES: &'static [Property] = &[];
    const ITEM: Option<u16> = Some(497);
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
    const COLLISION: PerState<ShapeId> = PerState::Uniform(ShapeId(120));
    const FACE_SUPPORT: PerState<FaceSupport> = PerState::Uniform(FaceSupport(0, 1, 0));
    type Behavior = crate::family::Plain;
}
