use crate::{
    Block, BlockId, BlockSettings, FaceSupport, PerState, Property, ShapeId, StateFlags, StateId,
};

pub struct WarpedNylium;

impl Block for WarpedNylium {
    const NAME: &'static str = "minecraft:warped_nylium";
    const ID: BlockId = BlockId(956);
    const FIRST_STATE: StateId = StateId(24314);
    const STATE_COUNT: u16 = 1;
    const DEFAULT_STATE: StateId = StateId(24314);
    const PROPERTIES: &'static [Property] = &[];
    const ITEM: Option<u16> = Some(61);
    const WALL_VARIANT: Option<BlockId> = None;
    const SETTINGS: BlockSettings = BlockSettings {
        hardness: 0.4,
        resistance: 0.4,
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
    const FLAGS: PerState<StateFlags> = PerState::Uniform(StateFlags(3614));
    const COLLISION: PerState<ShapeId> = PerState::Uniform(ShapeId(1));
    const FACE_SUPPORT: PerState<FaceSupport> = PerState::Uniform(FaceSupport(63, 63, 63));
    type Behavior = crate::family::Plain;
}
