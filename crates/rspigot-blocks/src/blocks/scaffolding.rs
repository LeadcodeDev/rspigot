use crate::{
    Block, BlockId, BlockSettings, FaceSupport, PerState, Property, ShapeId, StateFlags, StateId,
};

pub struct Scaffolding;

impl Block for Scaffolding {
    const NAME: &'static str = "minecraft:scaffolding";
    const ID: BlockId = BlockId(927);
    const FIRST_STATE: StateId = StateId(24063);
    const STATE_COUNT: u16 = 32;
    const DEFAULT_STATE: StateId = StateId(24094);
    const PROPERTIES: &'static [Property] = &[
        Property { name: "bottom", values: &["true", "false"] },
        Property { name: "distance", values: &["0", "1", "2", "3", "4", "5", "6", "7"] },
        Property { name: "waterlogged", values: &["true", "false"] },
    ];
    const ITEM: Option<u16> = Some(823);
    const WALL_VARIANT: Option<BlockId> = None;
    const SETTINGS: BlockSettings = BlockSettings {
        hardness: 0.0,
        resistance: 0.0,
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
        1, 0, 1, 0, 1, 0, 1, 0, 1, 0, 1, 0, 1, 0, 1, 0, 1, 0, 1, 0, 1, 0, 1, 0, 1, 0, 1, 0, 1, 0, 1,
        0,
    ]);
    const FLAGS: PerState<StateFlags> = PerState::Varies(&[
        StateFlags(132), StateFlags(36), StateFlags(132), StateFlags(36), StateFlags(132),
        StateFlags(36), StateFlags(132), StateFlags(36), StateFlags(132), StateFlags(36),
        StateFlags(132), StateFlags(36), StateFlags(132), StateFlags(36), StateFlags(132),
        StateFlags(36), StateFlags(132), StateFlags(36), StateFlags(132), StateFlags(36),
        StateFlags(132), StateFlags(36), StateFlags(132), StateFlags(36), StateFlags(132),
        StateFlags(36), StateFlags(132), StateFlags(36), StateFlags(132), StateFlags(36),
        StateFlags(132), StateFlags(36),
    ]);
    const COLLISION: PerState<ShapeId> = PerState::Uniform(ShapeId(275));
    const FACE_SUPPORT: PerState<FaceSupport> = PerState::Uniform(FaceSupport(2, 2, 2));
    type Behavior = crate::family::Scaffolding;
}
