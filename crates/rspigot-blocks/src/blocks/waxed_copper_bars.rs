use crate::{Block, BlockId, BlockSettings, PerState, Property, StateFlags, StateId};

pub struct WaxedCopperBars;

impl Block for WaxedCopperBars {
    const NAME: &'static str = "minecraft:waxed_copper_bars";
    const ID: BlockId = BlockId(396);
    const FIRST_STATE: StateId = StateId(9852);
    const STATE_COUNT: u16 = 32;
    const DEFAULT_STATE: StateId = StateId(9883);
    const PROPERTIES: &'static [Property] = &[
        Property { name: "east", values: &["true", "false"] },
        Property { name: "north", values: &["true", "false"] },
        Property { name: "south", values: &["true", "false"] },
        Property { name: "waterlogged", values: &["true", "false"] },
        Property { name: "west", values: &["true", "false"] },
    ];
    const ITEM: Option<u16> = Some(469);
    const WALL_VARIANT: Option<BlockId> = None;
    const SETTINGS: BlockSettings = BlockSettings {
        hardness: 5.0,
        resistance: 6.0,
        friction: 0.6,
        speed_factor: 1.0,
        jump_factor: 1.0,
        requires_correct_tool: true,
        ignited_by_lava: false,
    };
    const LIGHT_EMISSION: PerState<u8> = PerState::Uniform(0);
    const LIGHT_DAMPENING: PerState<u8> = PerState::Varies(&[
        1, 1, 0, 0, 1, 1, 0, 0, 1, 1, 0, 0, 1, 1, 0, 0, 1, 1, 0, 0, 1, 1, 0, 0, 1, 1, 0, 0, 1, 1, 0,
        0,
    ]);
    const FLAGS: PerState<StateFlags> = PerState::Varies(&[
        StateFlags(134), StateFlags(134), StateFlags(38), StateFlags(38), StateFlags(134),
        StateFlags(134), StateFlags(38), StateFlags(38), StateFlags(134), StateFlags(134),
        StateFlags(38), StateFlags(38), StateFlags(134), StateFlags(134), StateFlags(38),
        StateFlags(38), StateFlags(134), StateFlags(134), StateFlags(38), StateFlags(38),
        StateFlags(134), StateFlags(134), StateFlags(38), StateFlags(38), StateFlags(134),
        StateFlags(134), StateFlags(38), StateFlags(38), StateFlags(134), StateFlags(134),
        StateFlags(38), StateFlags(38),
    ]);
    type Behavior = crate::family::Plain;
}
