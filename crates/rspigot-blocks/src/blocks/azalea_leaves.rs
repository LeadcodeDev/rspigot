use crate::{
    Block, BlockId, BlockSettings, FaceSupport, PerState, Property, ShapeId, StateFlags, StateId,
};

pub struct AzaleaLeaves;

impl Block for AzaleaLeaves {
    const NAME: &'static str = "minecraft:azalea_leaves";
    const ID: BlockId = BlockId(106);
    const FIRST_STATE: StateId = StateId(603);
    const STATE_COUNT: u16 = 28;
    const DEFAULT_STATE: StateId = StateId(630);
    const PROPERTIES: &'static [Property] = &[
        Property { name: "distance", values: &["1", "2", "3", "4", "5", "6", "7"] },
        Property { name: "persistent", values: &["true", "false"] },
        Property { name: "waterlogged", values: &["true", "false"] },
    ];
    const ITEM: Option<u16> = Some(227);
    const WALL_VARIANT: Option<BlockId> = None;
    const SETTINGS: BlockSettings = BlockSettings {
        hardness: 0.2,
        resistance: 0.2,
        friction: 0.6,
        speed_factor: 1.0,
        jump_factor: 1.0,
        bounce_restitution: 0.0,
        fall_distance_reduction: 0.0,
        requires_correct_tool: false,
        ignited_by_lava: true,
    };
    const LIGHT_EMISSION: PerState<u8> = PerState::Uniform(0);
    const LIGHT_DAMPENING: PerState<u8> = PerState::Uniform(1);
    const FLAGS: PerState<StateFlags> = PerState::Varies(&[
        StateFlags(3206), StateFlags(3078), StateFlags(3206), StateFlags(3078), StateFlags(3206),
        StateFlags(3078), StateFlags(3206), StateFlags(3078), StateFlags(3206), StateFlags(3078),
        StateFlags(3206), StateFlags(3078), StateFlags(3206), StateFlags(3078), StateFlags(3206),
        StateFlags(3078), StateFlags(3206), StateFlags(3078), StateFlags(3206), StateFlags(3078),
        StateFlags(3206), StateFlags(3078), StateFlags(3206), StateFlags(3078), StateFlags(3206),
        StateFlags(3078), StateFlags(3718), StateFlags(3590),
    ]);
    const COLLISION: PerState<ShapeId> = PerState::Uniform(ShapeId(1));
    const FACE_SUPPORT: PerState<FaceSupport> = PerState::Uniform(FaceSupport(0, 0, 0));
    type Behavior = crate::family::Plain;
}
