use crate::{
    Block, BlockId, BlockSettings, FaceSupport, PerState, Property, ShapeId, StateFlags, StateId,
};

pub struct SuspiciousSand;

impl Block for SuspiciousSand {
    const NAME: &'static str = "minecraft:suspicious_sand";
    const ID: BlockId = BlockId(40);
    const FIRST_STATE: StateId = StateId(122);
    const STATE_COUNT: u16 = 4;
    const DEFAULT_STATE: StateId = StateId(122);
    const PROPERTIES: &'static [Property] = &[
        Property { name: "dusted", values: &["0", "1", "2", "3"] },
    ];
    const ITEM: Option<u16> = Some(89);
    const WALL_VARIANT: Option<BlockId> = None;
    const SETTINGS: BlockSettings = BlockSettings {
        hardness: 0.25,
        resistance: 0.25,
        friction: 0.6,
        speed_factor: 1.0,
        jump_factor: 1.0,
        bounce_restitution: 0.0,
        fall_distance_reduction: 0.0,
        requires_correct_tool: false,
        ignited_by_lava: false,
    };
    const LIGHT_EMISSION: PerState<u8> = PerState::Uniform(0);
    const LIGHT_DAMPENING: PerState<u8> = PerState::Uniform(15);
    const FLAGS: PerState<StateFlags> = PerState::Uniform(StateFlags(1054));
    const COLLISION: PerState<ShapeId> = PerState::Uniform(ShapeId(1));
    const FACE_SUPPORT: PerState<FaceSupport> = PerState::Uniform(FaceSupport(63, 63, 63));
    type Behavior = crate::family::Falling;
}
