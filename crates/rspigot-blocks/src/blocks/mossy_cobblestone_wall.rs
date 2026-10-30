use crate::{Block, BlockId, BlockSettings, PerState, Property, StateFlags, StateId};

pub struct MossyCobblestoneWall;

impl Block for MossyCobblestoneWall {
    const NAME: &'static str = "minecraft:mossy_cobblestone_wall";
    const ID: BlockId = BlockId(461);
    const FIRST_STATE: StateId = StateId(12047);
    const STATE_COUNT: u16 = 324;
    const DEFAULT_STATE: StateId = StateId(12050);
    const PROPERTIES: &'static [Property] = &[
        Property { name: "east", values: &["none", "low", "tall"] },
        Property { name: "north", values: &["none", "low", "tall"] },
        Property { name: "south", values: &["none", "low", "tall"] },
        Property { name: "up", values: &["true", "false"] },
        Property { name: "waterlogged", values: &["true", "false"] },
        Property { name: "west", values: &["none", "low", "tall"] },
    ];
    const ITEM: Option<u16> = Some(532);
    const WALL_VARIANT: Option<BlockId> = None;
    const SETTINGS: BlockSettings = BlockSettings {
        hardness: 2.0,
        resistance: 6.0,
        friction: 0.6,
        speed_factor: 1.0,
        jump_factor: 1.0,
        requires_correct_tool: true,
        ignited_by_lava: false,
    };
    const LIGHT_EMISSION: PerState<u8> = PerState::Uniform(0);
    const LIGHT_DAMPENING: PerState<u8> = PerState::Varies(&[
        1, 1, 1, 0, 0, 0, 1, 1, 1, 0, 0, 0, 1, 1, 1, 0, 0, 0, 1, 1, 1, 0, 0, 0, 1, 1, 1, 0, 0, 0, 1,
        1, 1, 0, 0, 0, 1, 1, 1, 0, 0, 0, 1, 1, 1, 0, 0, 0, 1, 1, 1, 0, 0, 0, 1, 1, 1, 0, 0, 0, 1, 1,
        1, 0, 0, 0, 1, 1, 1, 0, 0, 0, 1, 1, 1, 0, 0, 0, 1, 1, 1, 0, 0, 0, 1, 1, 1, 0, 0, 0, 1, 1, 1,
        0, 0, 0, 1, 1, 1, 0, 0, 0, 1, 1, 1, 0, 0, 0, 1, 1, 1, 0, 0, 0, 1, 1, 1, 0, 0, 0, 1, 1, 1, 0,
        0, 0, 1, 1, 1, 0, 0, 0, 1, 1, 1, 0, 0, 0, 1, 1, 1, 0, 0, 0, 1, 1, 1, 0, 0, 0, 1, 1, 1, 0, 0,
        0, 1, 1, 1, 0, 0, 0, 1, 1, 1, 0, 0, 0, 1, 1, 1, 0, 0, 0, 1, 1, 1, 0, 0, 0, 1, 1, 1, 0, 0, 0,
        1, 1, 1, 0, 0, 0, 1, 1, 1, 0, 0, 0, 1, 1, 1, 0, 0, 0, 1, 1, 1, 0, 0, 0, 1, 1, 1, 0, 0, 0, 1,
        1, 1, 0, 0, 0, 1, 1, 1, 0, 0, 0, 1, 1, 1, 0, 0, 0, 1, 1, 1, 0, 0, 0, 1, 1, 1, 0, 0, 0, 1, 1,
        1, 0, 0, 0, 1, 1, 1, 0, 0, 0, 1, 1, 1, 0, 0, 0, 1, 1, 1, 0, 0, 0, 1, 1, 1, 0, 0, 0, 1, 1, 1,
        0, 0, 0, 1, 1, 1, 0, 0, 0, 1, 1, 1, 0, 0, 0, 1, 1, 1, 0, 0, 0, 1, 1, 1, 0, 0, 0, 1, 1, 1, 0,
        0, 0, 1, 1, 1, 0, 0, 0, 1, 1, 1, 0, 0, 0,
    ]);
    const FLAGS: PerState<StateFlags> = PerState::Varies(&[
        StateFlags(142), StateFlags(142), StateFlags(142), StateFlags(46), StateFlags(46),
        StateFlags(46), StateFlags(138), StateFlags(142), StateFlags(142), StateFlags(42),
        StateFlags(46), StateFlags(46), StateFlags(142), StateFlags(142), StateFlags(142),
        StateFlags(46), StateFlags(46), StateFlags(46), StateFlags(142), StateFlags(142),
        StateFlags(142), StateFlags(46), StateFlags(46), StateFlags(46), StateFlags(142),
        StateFlags(142), StateFlags(142), StateFlags(46), StateFlags(46), StateFlags(46),
        StateFlags(142), StateFlags(142), StateFlags(142), StateFlags(46), StateFlags(46),
        StateFlags(46), StateFlags(142), StateFlags(142), StateFlags(142), StateFlags(46),
        StateFlags(46), StateFlags(46), StateFlags(142), StateFlags(142), StateFlags(142),
        StateFlags(46), StateFlags(46), StateFlags(46), StateFlags(142), StateFlags(142),
        StateFlags(142), StateFlags(46), StateFlags(46), StateFlags(46), StateFlags(142),
        StateFlags(142), StateFlags(142), StateFlags(46), StateFlags(46), StateFlags(46),
        StateFlags(142), StateFlags(142), StateFlags(142), StateFlags(46), StateFlags(46),
        StateFlags(46), StateFlags(142), StateFlags(142), StateFlags(142), StateFlags(46),
        StateFlags(46), StateFlags(46), StateFlags(142), StateFlags(142), StateFlags(142),
        StateFlags(46), StateFlags(46), StateFlags(46), StateFlags(142), StateFlags(142),
        StateFlags(142), StateFlags(46), StateFlags(46), StateFlags(46), StateFlags(142),
        StateFlags(142), StateFlags(142), StateFlags(46), StateFlags(46), StateFlags(46),
        StateFlags(142), StateFlags(142), StateFlags(142), StateFlags(46), StateFlags(46),
        StateFlags(46), StateFlags(142), StateFlags(142), StateFlags(142), StateFlags(46),
        StateFlags(46), StateFlags(46), StateFlags(142), StateFlags(142), StateFlags(142),
        StateFlags(46), StateFlags(46), StateFlags(46), StateFlags(142), StateFlags(142),
        StateFlags(142), StateFlags(46), StateFlags(46), StateFlags(46), StateFlags(142),
        StateFlags(142), StateFlags(142), StateFlags(46), StateFlags(46), StateFlags(46),
        StateFlags(142), StateFlags(142), StateFlags(142), StateFlags(46), StateFlags(46),
        StateFlags(46), StateFlags(142), StateFlags(142), StateFlags(142), StateFlags(46),
        StateFlags(46), StateFlags(46), StateFlags(142), StateFlags(142), StateFlags(142),
        StateFlags(46), StateFlags(46), StateFlags(46), StateFlags(142), StateFlags(142),
        StateFlags(142), StateFlags(46), StateFlags(46), StateFlags(46), StateFlags(142),
        StateFlags(142), StateFlags(142), StateFlags(46), StateFlags(46), StateFlags(46),
        StateFlags(142), StateFlags(142), StateFlags(142), StateFlags(46), StateFlags(46),
        StateFlags(46), StateFlags(142), StateFlags(142), StateFlags(142), StateFlags(46),
        StateFlags(46), StateFlags(46), StateFlags(142), StateFlags(142), StateFlags(142),
        StateFlags(46), StateFlags(46), StateFlags(46), StateFlags(142), StateFlags(142),
        StateFlags(142), StateFlags(46), StateFlags(46), StateFlags(46), StateFlags(142),
        StateFlags(142), StateFlags(142), StateFlags(46), StateFlags(46), StateFlags(46),
        StateFlags(142), StateFlags(142), StateFlags(142), StateFlags(46), StateFlags(46),
        StateFlags(46), StateFlags(142), StateFlags(142), StateFlags(142), StateFlags(46),
        StateFlags(46), StateFlags(46), StateFlags(142), StateFlags(142), StateFlags(142),
        StateFlags(46), StateFlags(46), StateFlags(46), StateFlags(142), StateFlags(142),
        StateFlags(142), StateFlags(46), StateFlags(46), StateFlags(46), StateFlags(142),
        StateFlags(142), StateFlags(142), StateFlags(46), StateFlags(46), StateFlags(46),
        StateFlags(142), StateFlags(142), StateFlags(142), StateFlags(46), StateFlags(46),
        StateFlags(46), StateFlags(142), StateFlags(142), StateFlags(142), StateFlags(46),
        StateFlags(46), StateFlags(46), StateFlags(142), StateFlags(142), StateFlags(142),
        StateFlags(46), StateFlags(46), StateFlags(46), StateFlags(142), StateFlags(142),
        StateFlags(142), StateFlags(46), StateFlags(46), StateFlags(46), StateFlags(142),
        StateFlags(142), StateFlags(142), StateFlags(46), StateFlags(46), StateFlags(46),
        StateFlags(142), StateFlags(142), StateFlags(142), StateFlags(46), StateFlags(46),
        StateFlags(46), StateFlags(142), StateFlags(142), StateFlags(142), StateFlags(46),
        StateFlags(46), StateFlags(46), StateFlags(142), StateFlags(142), StateFlags(142),
        StateFlags(46), StateFlags(46), StateFlags(46), StateFlags(142), StateFlags(142),
        StateFlags(142), StateFlags(46), StateFlags(46), StateFlags(46), StateFlags(142),
        StateFlags(142), StateFlags(142), StateFlags(46), StateFlags(46), StateFlags(46),
        StateFlags(142), StateFlags(142), StateFlags(142), StateFlags(46), StateFlags(46),
        StateFlags(46), StateFlags(142), StateFlags(142), StateFlags(142), StateFlags(46),
        StateFlags(46), StateFlags(46), StateFlags(142), StateFlags(142), StateFlags(142),
        StateFlags(46), StateFlags(46), StateFlags(46), StateFlags(142), StateFlags(142),
        StateFlags(142), StateFlags(46), StateFlags(46), StateFlags(46), StateFlags(142),
        StateFlags(142), StateFlags(142), StateFlags(46), StateFlags(46), StateFlags(46),
        StateFlags(142), StateFlags(142), StateFlags(142), StateFlags(46), StateFlags(46),
        StateFlags(46), StateFlags(142), StateFlags(142), StateFlags(142), StateFlags(46),
        StateFlags(46), StateFlags(46), StateFlags(142), StateFlags(142), StateFlags(142),
        StateFlags(46), StateFlags(46), StateFlags(46), StateFlags(142), StateFlags(142),
        StateFlags(142), StateFlags(46), StateFlags(46), StateFlags(46),
    ]);
    type Behavior = crate::family::Plain;
}
