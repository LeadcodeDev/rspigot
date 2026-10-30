use crate::{Block, BlockId, BlockSettings, PerState, Property, StateFlags, StateId};

pub struct BirchShelf;

impl Block for BirchShelf {
    const NAME: &'static str = "minecraft:birch_shelf";
    const ID: BlockId = BlockId(225);
    const FIRST_STATE: StateId = StateId(4220);
    const STATE_COUNT: u16 = 64;
    const DEFAULT_STATE: StateId = StateId(4229);
    const PROPERTIES: &'static [Property] = &[
        Property { name: "facing", values: &["north", "south", "west", "east"] },
        Property { name: "powered", values: &["true", "false"] },
        Property { name: "side_chain", values: &["unconnected", "right", "center", "left"] },
        Property { name: "waterlogged", values: &["true", "false"] },
    ];
    const ITEM: Option<u16> = Some(379);
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
        1, 0, 1, 0, 1, 0, 1, 0, 1, 0, 1, 0, 1, 0, 1, 0, 1, 0, 1, 0, 1, 0, 1, 0, 1, 0, 1, 0, 1, 0, 1,
        0, 1, 0, 1, 0, 1, 0, 1, 0, 1, 0, 1, 0, 1, 0, 1, 0, 1, 0, 1, 0, 1, 0, 1, 0, 1, 0, 1, 0, 1, 0,
        1, 0,
    ]);
    const FLAGS: PerState<StateFlags> = PerState::Varies(&[
        StateFlags(142), StateFlags(46), StateFlags(142), StateFlags(46), StateFlags(142),
        StateFlags(46), StateFlags(142), StateFlags(46), StateFlags(142), StateFlags(46),
        StateFlags(142), StateFlags(46), StateFlags(142), StateFlags(46), StateFlags(142),
        StateFlags(46), StateFlags(142), StateFlags(46), StateFlags(142), StateFlags(46),
        StateFlags(142), StateFlags(46), StateFlags(142), StateFlags(46), StateFlags(142),
        StateFlags(46), StateFlags(142), StateFlags(46), StateFlags(142), StateFlags(46),
        StateFlags(142), StateFlags(46), StateFlags(142), StateFlags(46), StateFlags(142),
        StateFlags(46), StateFlags(142), StateFlags(46), StateFlags(142), StateFlags(46),
        StateFlags(142), StateFlags(46), StateFlags(142), StateFlags(46), StateFlags(142),
        StateFlags(46), StateFlags(142), StateFlags(46), StateFlags(142), StateFlags(46),
        StateFlags(142), StateFlags(46), StateFlags(142), StateFlags(46), StateFlags(142),
        StateFlags(46), StateFlags(142), StateFlags(46), StateFlags(142), StateFlags(46),
        StateFlags(142), StateFlags(46), StateFlags(142), StateFlags(46),
    ]);
    type Behavior = crate::family::HorizontalAway;
}
