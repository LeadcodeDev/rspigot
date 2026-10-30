use crate::{Block, BlockId, BlockSettings, PerState, Property, StateFlags, StateId};

pub struct BigDripleaf;

impl Block for BigDripleaf {
    const NAME: &'static str = "minecraft:big_dripleaf";
    const ID: BlockId = BlockId(1235);
    const FIRST_STATE: StateId = StateId(33713);
    const STATE_COUNT: u16 = 32;
    const DEFAULT_STATE: StateId = StateId(33714);
    const PROPERTIES: &'static [Property] = &[
        Property { name: "facing", values: &["north", "south", "west", "east"] },
        Property { name: "tilt", values: &["none", "unstable", "partial", "full"] },
        Property { name: "waterlogged", values: &["true", "false"] },
    ];
    const ITEM: Option<u16> = Some(338);
    const WALL_VARIANT: Option<BlockId> = None;
    const SETTINGS: BlockSettings = BlockSettings {
        hardness: 0.1,
        resistance: 0.1,
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
        StateFlags(140), StateFlags(44), StateFlags(140), StateFlags(44), StateFlags(140),
        StateFlags(44), StateFlags(136), StateFlags(40), StateFlags(140), StateFlags(44),
        StateFlags(140), StateFlags(44), StateFlags(140), StateFlags(44), StateFlags(136),
        StateFlags(40), StateFlags(140), StateFlags(44), StateFlags(140), StateFlags(44),
        StateFlags(140), StateFlags(44), StateFlags(136), StateFlags(40), StateFlags(140),
        StateFlags(44), StateFlags(140), StateFlags(44), StateFlags(140), StateFlags(44),
        StateFlags(136), StateFlags(40),
    ]);
    type Behavior = crate::family::Plain;
}
