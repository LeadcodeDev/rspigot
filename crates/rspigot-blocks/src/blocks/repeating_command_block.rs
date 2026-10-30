use crate::{
    Block, BlockId, BlockSettings, FaceSupport, PerState, Property, ShapeId, StateFlags, StateId,
};

pub struct RepeatingCommandBlock;

impl Block for RepeatingCommandBlock {
    const NAME: &'static str = "minecraft:repeating_command_block";
    const ID: BlockId = BlockId(726);
    const FIRST_STATE: StateId = StateId(16798);
    const STATE_COUNT: u16 = 12;
    const DEFAULT_STATE: StateId = StateId(16804);
    const PROPERTIES: &'static [Property] = &[
        Property { name: "conditional", values: &["true", "false"] },
        Property { name: "facing", values: &["north", "east", "south", "west", "up", "down"] },
    ];
    const ITEM: Option<u16> = Some(648);
    const WALL_VARIANT: Option<BlockId> = None;
    const SETTINGS: BlockSettings = BlockSettings {
        hardness: -1.0,
        resistance: 3600000.0,
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
    const FLAGS: PerState<StateFlags> = PerState::Uniform(StateFlags(3102));
    const COLLISION: PerState<ShapeId> = PerState::Uniform(ShapeId(1));
    const FACE_SUPPORT: PerState<FaceSupport> = PerState::Uniform(FaceSupport(63, 63, 63));
    type Behavior = crate::family::Directional<true>;
}
