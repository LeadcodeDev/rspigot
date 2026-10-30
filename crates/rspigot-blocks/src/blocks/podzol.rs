use crate::{
    Block, BlockId, BlockSettings, FaceSupport, PerState, Property, ShapeId, StateFlags, StateId,
};

pub struct Podzol;

impl Block for Podzol {
    const NAME: &'static str = "minecraft:podzol";
    const ID: BlockId = BlockId(11);
    const FIRST_STATE: StateId = StateId(12);
    const STATE_COUNT: u16 = 2;
    const DEFAULT_STATE: StateId = StateId(13);
    const PROPERTIES: &'static [Property] = &[
        Property { name: "snowy", values: &["true", "false"] },
    ];
    const ITEM: Option<u16> = Some(57);
    const WALL_VARIANT: Option<BlockId> = None;
    const SETTINGS: BlockSettings = BlockSettings {
        hardness: 0.5,
        resistance: 0.5,
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
    const FLAGS: PerState<StateFlags> = PerState::Uniform(StateFlags(3102));
    const COLLISION: PerState<ShapeId> = PerState::Uniform(ShapeId(1));
    const FACE_SUPPORT: PerState<FaceSupport> = PerState::Uniform(FaceSupport(63, 63, 63));
    type Behavior = crate::family::Plain;
}
