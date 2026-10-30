use crate::{
    Block, BlockId, BlockSettings, FaceSupport, PerState, Property, ShapeId, StateFlags, StateId,
};

pub struct SmallDripleaf;

impl Block for SmallDripleaf {
    const NAME: &'static str = "minecraft:small_dripleaf";
    const ID: BlockId = BlockId(1237);
    const FIRST_STATE: StateId = StateId(33753);
    const STATE_COUNT: u16 = 16;
    const DEFAULT_STATE: StateId = StateId(33756);
    const PROPERTIES: &'static [Property] = &[
        Property { name: "facing", values: &["north", "south", "west", "east"] },
        Property { name: "half", values: &["upper", "lower"] },
        Property { name: "waterlogged", values: &["true", "false"] },
    ];
    const ITEM: Option<u16> = Some(339);
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
        1, 0, 1, 0, 1, 0, 1, 0, 1, 0, 1, 0, 1, 0, 1, 0,
    ]);
    const FLAGS: PerState<StateFlags> = PerState::Varies(&[
        StateFlags(16512), StateFlags(16416), StateFlags(16512), StateFlags(16416),
        StateFlags(16512), StateFlags(16416), StateFlags(16512), StateFlags(16416),
        StateFlags(16512), StateFlags(16416), StateFlags(16512), StateFlags(16416),
        StateFlags(16512), StateFlags(16416), StateFlags(16512), StateFlags(16416),
    ]);
    const COLLISION: PerState<ShapeId> = PerState::Uniform(ShapeId(0));
    const FACE_SUPPORT: PerState<FaceSupport> = PerState::Uniform(FaceSupport(0, 0, 0));
    type Behavior = crate::family::Plain;
}
