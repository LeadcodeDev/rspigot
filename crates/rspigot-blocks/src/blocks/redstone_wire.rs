use crate::{Block, BlockId, BlockSettings, PerState, Property, StateFlags, StateId};

pub struct RedstoneWire;

impl Block for RedstoneWire {
    const NAME: &'static str = "minecraft:redstone_wire";
    const ID: BlockId = BlockId(246);
    const FIRST_STATE: StateId = StateId(5567);
    const STATE_COUNT: u16 = 1296;
    const DEFAULT_STATE: StateId = StateId(6727);
    const PROPERTIES: &'static [Property] = &[
        Property { name: "east", values: &["up", "side", "none"] },
        Property { name: "north", values: &["up", "side", "none"] },
        Property {
            name: "power",
            values: &[
                "0", "1", "2", "3", "4", "5", "6", "7", "8", "9", "10", "11", "12", "13", "14",
                "15",
            ],
        },
        Property { name: "south", values: &["up", "side", "none"] },
        Property { name: "west", values: &["up", "side", "none"] },
    ];
    const ITEM: Option<u16> = Some(824);
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
    const LIGHT_DAMPENING: PerState<u8> = PerState::Uniform(0);
    const FLAGS: PerState<StateFlags> = PerState::Uniform(StateFlags(32));
    type Behavior = crate::family::Plain;
}
