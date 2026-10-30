use crate::{
    Block, BlockId, BlockSettings, FaceSupport, PerState, Property, ShapeId, StateFlags, StateId,
};

pub struct MangroveFenceGate;

impl Block for MangroveFenceGate {
    const NAME: &'static str = "minecraft:mangrove_fence_gate";
    const ID: BlockId = BlockId(689);
    const FIRST_STATE: StateId = StateId(15555);
    const STATE_COUNT: u16 = 32;
    const DEFAULT_STATE: StateId = StateId(15562);
    const PROPERTIES: &'static [Property] = &[
        Property { name: "facing", values: &["north", "south", "west", "east"] },
        Property { name: "in_wall", values: &["true", "false"] },
        Property { name: "open", values: &["true", "false"] },
        Property { name: "powered", values: &["true", "false"] },
    ];
    const ITEM: Option<u16> = Some(940);
    const WALL_VARIANT: Option<BlockId> = None;
    const SETTINGS: BlockSettings = BlockSettings {
        hardness: 2.0,
        resistance: 3.0,
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
    const FLAGS: PerState<StateFlags> = PerState::Varies(&[
        StateFlags(18474), StateFlags(18474), StateFlags(2094), StateFlags(2094), StateFlags(18474),
        StateFlags(18474), StateFlags(2094), StateFlags(2094), StateFlags(18474), StateFlags(18474),
        StateFlags(2094), StateFlags(2094), StateFlags(18474), StateFlags(18474), StateFlags(2094),
        StateFlags(2094), StateFlags(18474), StateFlags(18474), StateFlags(2094), StateFlags(2094),
        StateFlags(18474), StateFlags(18474), StateFlags(2094), StateFlags(2094), StateFlags(18474),
        StateFlags(18474), StateFlags(2094), StateFlags(2094), StateFlags(18474), StateFlags(18474),
        StateFlags(2094), StateFlags(2094),
    ]);
    const COLLISION: PerState<ShapeId> = PerState::Varies(crate::shapes::COLLISION_OAK_FENCE_GATE);
    const FACE_SUPPORT: PerState<FaceSupport> =
        PerState::Varies(crate::shapes::SUPPORT_OAK_FENCE_GATE);
    type Behavior = crate::family::FenceGate;
}
