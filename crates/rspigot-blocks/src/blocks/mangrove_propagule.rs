use crate::{Block, BlockId, BlockSettings, PerState, Property, StateFlags, StateId};

pub struct MangrovePropagule;

impl Block for MangrovePropagule {
    const NAME: &'static str = "minecraft:mangrove_propagule";
    const ID: BlockId = BlockId(34);
    const FIRST_STATE: StateId = StateId(46);
    const STATE_COUNT: u16 = 40;
    const DEFAULT_STATE: StateId = StateId(51);
    const PROPERTIES: &'static [Property] = &[
        Property { name: "age", values: &["0", "1", "2", "3", "4"] },
        Property { name: "hanging", values: &["true", "false"] },
        Property { name: "stage", values: &["0", "1"] },
        Property { name: "waterlogged", values: &["true", "false"] },
    ];
    const ITEM: Option<u16> = Some(85);
    const WALL_VARIANT: Option<BlockId> = None;
    const SETTINGS: BlockSettings = BlockSettings {
        hardness: 0.0,
        resistance: 0.0,
        friction: 0.6,
        speed_factor: 1.0,
        jump_factor: 1.0,
        requires_correct_tool: false,
        ignited_by_lava: false,
    };
    const LIGHT_EMISSION: PerState<u8> = PerState::Uniform(0);
    const LIGHT_DAMPENING: PerState<u8> = PerState::Varies(&[
        1, 0, 1, 0, 1, 0, 1, 0, 1, 0, 1, 0, 1, 0, 1, 0, 1, 0, 1, 0, 1, 0, 1, 0, 1, 0, 1, 0, 1, 0, 1,
        0, 1, 0, 1, 0, 1, 0, 1, 0,
    ]);
    const FLAGS: PerState<StateFlags> = PerState::Varies(&[
        StateFlags(640), StateFlags(544), StateFlags(640), StateFlags(544), StateFlags(640),
        StateFlags(544), StateFlags(640), StateFlags(544), StateFlags(640), StateFlags(544),
        StateFlags(640), StateFlags(544), StateFlags(640), StateFlags(544), StateFlags(640),
        StateFlags(544), StateFlags(640), StateFlags(544), StateFlags(640), StateFlags(544),
        StateFlags(640), StateFlags(544), StateFlags(640), StateFlags(544), StateFlags(640),
        StateFlags(544), StateFlags(640), StateFlags(544), StateFlags(640), StateFlags(544),
        StateFlags(640), StateFlags(544), StateFlags(640), StateFlags(544), StateFlags(640),
        StateFlags(544), StateFlags(640), StateFlags(544), StateFlags(640), StateFlags(544),
    ]);
    type Behavior = crate::family::Plain;
}
