use crate::{
    Block, BlockId, BlockSettings, FaceSupport, PerState, Property, ShapeId, StateFlags, StateId,
};

pub struct Barrier;

impl Block for Barrier {
    const NAME: &'static str = "minecraft:barrier";
    const ID: BlockId = BlockId(578);
    const FIRST_STATE: StateId = StateId(14380);
    const STATE_COUNT: u16 = 2;
    const DEFAULT_STATE: StateId = StateId(14381);
    const PROPERTIES: &'static [Property] = &[
        Property { name: "waterlogged", values: &["true", "false"] },
    ];
    const ITEM: Option<u16> = Some(577);
    const WALL_VARIANT: Option<BlockId> = None;
    const SETTINGS: BlockSettings = BlockSettings {
        hardness: -1.0,
        resistance: 3600000.8,
        friction: 0.6,
        speed_factor: 1.0,
        jump_factor: 1.0,
        bounce_restitution: 0.0,
        fall_distance_reduction: 0.0,
        requires_correct_tool: false,
        ignited_by_lava: false,
    };
    const LIGHT_EMISSION: PerState<u8> = PerState::Uniform(0);
    const LIGHT_DAMPENING: PerState<u8> = PerState::Varies(&[
        1, 0,
    ]);
    const FLAGS: PerState<StateFlags> = PerState::Varies(&[
        StateFlags(3206), StateFlags(3110),
    ]);
    const COLLISION: PerState<ShapeId> = PerState::Uniform(ShapeId(1));
    const FACE_SUPPORT: PerState<FaceSupport> = PerState::Uniform(FaceSupport(63, 63, 63));
    type Behavior = crate::family::Plain;
}
