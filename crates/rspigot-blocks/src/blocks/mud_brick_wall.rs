use crate::{Block, BlockId, BlockSettings, PerState, Property, StateFlags, StateId};

pub struct MudBrickWall;

impl Block for MudBrickWall {
    const NAME: &'static str = "minecraft:mud_brick_wall";
    const ID: BlockId = BlockId(920);
    const FIRST_STATE: StateId = StateId(21795);
    const STATE_COUNT: u16 = 324;
    const DEFAULT_STATE: StateId = StateId(21798);
    const PROPERTIES: &'static [Property] = &[
        Property { name: "east", values: &["none", "low", "tall"] },
        Property { name: "north", values: &["none", "low", "tall"] },
        Property { name: "south", values: &["none", "low", "tall"] },
        Property { name: "up", values: &["true", "false"] },
        Property { name: "waterlogged", values: &["true", "false"] },
        Property { name: "west", values: &["none", "low", "tall"] },
    ];
    const ITEM: Option<u16> = Some(539);
    const WALL_VARIANT: Option<BlockId> = None;
    const SETTINGS: BlockSettings = BlockSettings {
        hardness: 1.5,
        resistance: 3.0,
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
