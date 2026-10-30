use crate::{
    Block, BlockId, BlockSettings, FaceSupport, PerState, Property, ShapeId, StateFlags, StateId,
};

pub struct MagentaCarpet;

impl Block for MagentaCarpet {
    const NAME: &'static str = "minecraft:magenta_carpet";
    const ID: BlockId = BlockId(594);
    const FIRST_STATE: StateId = StateId(14745);
    const STATE_COUNT: u16 = 1;
    const DEFAULT_STATE: StateId = StateId(14745);
    const PROPERTIES: &'static [Property] = &[];
    const ITEM: Option<u16> = Some(582);
    const WALL_VARIANT: Option<BlockId> = None;
    const SETTINGS: BlockSettings = BlockSettings {
        hardness: 0.1,
        resistance: 0.1,
        friction: 0.6,
        speed_factor: 1.0,
        jump_factor: 1.0,
        bounce_restitution: 0.0,
        fall_distance_reduction: 0.0,
        requires_correct_tool: false,
        ignited_by_lava: true,
    };
    const LIGHT_EMISSION: PerState<u8> = PerState::Uniform(0);
    const LIGHT_DAMPENING: PerState<u8> = PerState::Uniform(0);
    const FLAGS: PerState<StateFlags> = PerState::Uniform(StateFlags(16428));
    const COLLISION: PerState<ShapeId> = PerState::Uniform(ShapeId(195));
    const FACE_SUPPORT: PerState<FaceSupport> = PerState::Uniform(FaceSupport(1, 1, 1));
    type Behavior = crate::family::Plain;
}
