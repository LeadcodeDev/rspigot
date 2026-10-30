use crate::{Block, BlockId, BlockSettings, PerState, Property, StateFlags, StateId};

pub struct PoplarFence;

impl Block for PoplarFence {
    const NAME: &'static str = "minecraft:poplar_fence";
    const ID: BlockId = BlockId(700);
    const FIRST_STATE: StateId = StateId(15907);
    const STATE_COUNT: u16 = 32;
    const DEFAULT_STATE: StateId = StateId(15938);
    const PROPERTIES: &'static [Property] = &[
        Property { name: "east", values: &["true", "false"] },
        Property { name: "north", values: &["true", "false"] },
        Property { name: "south", values: &["true", "false"] },
        Property { name: "waterlogged", values: &["true", "false"] },
        Property { name: "west", values: &["true", "false"] },
    ];
    const ITEM: Option<u16> = Some(426);
    const WALL_VARIANT: Option<BlockId> = None;
    const SETTINGS: BlockSettings = BlockSettings {
        hardness: 2.0,
        resistance: 3.0,
        friction: 0.6,
        speed_factor: 1.0,
        jump_factor: 1.0,
        requires_correct_tool: false,
        ignited_by_lava: true,
    };
    const LIGHT_EMISSION: PerState<u8> = PerState::Uniform(0);
    const LIGHT_DAMPENING: PerState<u8> = PerState::Varies(&[
        1, 1, 0, 0, 1, 1, 0, 0, 1, 1, 0, 0, 1, 1, 0, 0, 1, 1, 0, 0, 1, 1, 0, 0, 1, 1, 0, 0, 1, 1, 0,
        0,
    ]);
    const FLAGS: PerState<StateFlags> = PerState::Varies(&[
        StateFlags(142), StateFlags(142), StateFlags(46), StateFlags(46), StateFlags(142),
        StateFlags(142), StateFlags(46), StateFlags(46), StateFlags(142), StateFlags(142),
        StateFlags(46), StateFlags(46), StateFlags(142), StateFlags(142), StateFlags(46),
        StateFlags(46), StateFlags(142), StateFlags(142), StateFlags(46), StateFlags(46),
        StateFlags(142), StateFlags(142), StateFlags(46), StateFlags(46), StateFlags(142),
        StateFlags(142), StateFlags(46), StateFlags(46), StateFlags(142), StateFlags(142),
        StateFlags(46), StateFlags(46),
    ]);
    type Behavior = crate::family::Plain;
}
