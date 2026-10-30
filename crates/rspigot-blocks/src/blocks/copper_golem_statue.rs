use crate::{Block, BlockId, BlockSettings, PerState, Property, StateFlags, StateId};

pub struct CopperGolemStatue;

impl Block for CopperGolemStatue {
    const NAME: &'static str = "minecraft:copper_golem_statue";
    const ID: BlockId = BlockId(1206);
    const FIRST_STATE: StateId = StateId(33117);
    const STATE_COUNT: u16 = 32;
    const DEFAULT_STATE: StateId = StateId(33118);
    const PROPERTIES: &'static [Property] = &[
        Property { name: "copper_golem_pose", values: &["standing", "sitting", "running", "star"] },
        Property { name: "facing", values: &["north", "south", "west", "east"] },
        Property { name: "waterlogged", values: &["true", "false"] },
    ];
    const ITEM: Option<u16> = Some(1645);
    const WALL_VARIANT: Option<BlockId> = None;
    const SETTINGS: BlockSettings = BlockSettings {
        hardness: 3.0,
        resistance: 6.0,
        friction: 0.6,
        speed_factor: 1.0,
        jump_factor: 1.0,
        requires_correct_tool: false,
        ignited_by_lava: false,
    };
    const LIGHT_EMISSION: PerState<u8> = PerState::Uniform(0);
    const LIGHT_DAMPENING: PerState<u8> = PerState::Varies(&[
        1, 0, 1, 0, 1, 0, 1, 0, 1, 0, 1, 0, 1, 0, 1, 0, 1, 0, 1, 0, 1, 0, 1, 0, 1, 0, 1, 0, 1, 0, 1,
        0,
    ]);
    const FLAGS: PerState<StateFlags> = PerState::Varies(&[
        StateFlags(644), StateFlags(548), StateFlags(644), StateFlags(548), StateFlags(644),
        StateFlags(548), StateFlags(644), StateFlags(548), StateFlags(644), StateFlags(548),
        StateFlags(644), StateFlags(548), StateFlags(644), StateFlags(548), StateFlags(644),
        StateFlags(548), StateFlags(644), StateFlags(548), StateFlags(644), StateFlags(548),
        StateFlags(644), StateFlags(548), StateFlags(644), StateFlags(548), StateFlags(644),
        StateFlags(548), StateFlags(644), StateFlags(548), StateFlags(644), StateFlags(548),
        StateFlags(644), StateFlags(548),
    ]);
    type Behavior = crate::family::HorizontalAway;
}
