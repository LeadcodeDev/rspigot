use crate::{Block, BlockId, BlockSettings, PerState, Property, StateFlags, StateId};

pub struct Fire;

impl Block for Fire {
    const NAME: &'static str = "minecraft:fire";
    const ID: BlockId = BlockId(240);
    const FIRST_STATE: StateId = StateId(4931);
    const STATE_COUNT: u16 = 512;
    const DEFAULT_STATE: StateId = StateId(4962);
    const PROPERTIES: &'static [Property] = &[
        Property {
            name: "age",
            values: &[
                "0", "1", "2", "3", "4", "5", "6", "7", "8", "9", "10", "11", "12", "13", "14",
                "15",
            ],
        },
        Property { name: "east", values: &["true", "false"] },
        Property { name: "north", values: &["true", "false"] },
        Property { name: "south", values: &["true", "false"] },
        Property { name: "up", values: &["true", "false"] },
        Property { name: "west", values: &["true", "false"] },
    ];
    const ITEM: Option<u16> = None;
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
    const LIGHT_EMISSION: PerState<u8> = PerState::Uniform(15);
    const LIGHT_DAMPENING: PerState<u8> = PerState::Uniform(0);
    const FLAGS: PerState<StateFlags> = PerState::Uniform(StateFlags(96));
    type Behavior = crate::family::Plain;
}
