use crate::{
    Block, BlockId, BlockSettings, FaceSupport, PerState, Property, ShapeId, StateFlags, StateId,
};

pub struct Sandstone;

impl Block for Sandstone {
    const NAME: &'static str = "minecraft:sandstone";
    const ID: BlockId = BlockId(115);
    const FIRST_STATE: StateId = StateId(677);
    const STATE_COUNT: u16 = 1;
    const DEFAULT_STATE: StateId = StateId(677);
    const PROPERTIES: &'static [Property] = &[];
    const ITEM: Option<u16> = Some(234);
    const WALL_VARIANT: Option<BlockId> = None;
    const SETTINGS: BlockSettings = BlockSettings {
        hardness: 0.8,
        resistance: 0.8,
        friction: 0.6,
        speed_factor: 1.0,
        jump_factor: 1.0,
        bounce_restitution: 0.0,
        fall_distance_reduction: 0.0,
        requires_correct_tool: true,
        ignited_by_lava: false,
    };
    const LIGHT_EMISSION: PerState<u8> = PerState::Uniform(0);
    const LIGHT_DAMPENING: PerState<u8> = PerState::Uniform(15);
    const FLAGS: PerState<StateFlags> = PerState::Uniform(StateFlags(7198));
    const COLLISION: PerState<ShapeId> = PerState::Uniform(ShapeId(1));
    const FACE_SUPPORT: PerState<FaceSupport> = PerState::Uniform(FaceSupport(63, 63, 63));
    type Behavior = crate::family::Plain;
}
