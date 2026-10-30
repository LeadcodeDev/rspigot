use crate::{Block, BlockId, BlockSettings, PerState, Property, StateFlags, StateId};

pub struct Bell;

impl Block for Bell {
    const NAME: &'static str = "minecraft:bell";
    const ID: BlockId = BlockId(938);
    const FIRST_STATE: StateId = StateId(24162);
    const STATE_COUNT: u16 = 32;
    const DEFAULT_STATE: StateId = StateId(24163);
    const PROPERTIES: &'static [Property] = &[
        Property {
            name: "attachment",
            values: &[
                "floor", "ceiling", "single_wall", "double_wall",
            ],
        },
        Property { name: "facing", values: &["north", "south", "west", "east"] },
        Property { name: "powered", values: &["true", "false"] },
    ];
    const ITEM: Option<u16> = Some(1514);
    const WALL_VARIANT: Option<BlockId> = None;
    const SETTINGS: BlockSettings = BlockSettings {
        hardness: 5.0,
        resistance: 5.0,
        friction: 0.6,
        speed_factor: 1.0,
        jump_factor: 1.0,
        requires_correct_tool: false,
        ignited_by_lava: false,
    };
    const LIGHT_EMISSION: PerState<u8> = PerState::Uniform(0);
    const LIGHT_DAMPENING: PerState<u8> = PerState::Uniform(0);
    const FLAGS: PerState<StateFlags> = PerState::Uniform(StateFlags(46));
    type Behavior = crate::family::Plain;
}
