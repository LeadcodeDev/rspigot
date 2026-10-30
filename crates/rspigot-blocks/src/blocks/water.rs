use crate::{Block, BlockId, BlockSettings, PerState, Property, StateFlags, StateId};

pub struct Water;

impl Block for Water {
    const NAME: &'static str = "minecraft:water";
    const ID: BlockId = BlockId(37);
    const FIRST_STATE: StateId = StateId(89);
    const STATE_COUNT: u16 = 16;
    const DEFAULT_STATE: StateId = StateId(89);
    const PROPERTIES: &'static [Property] = &[
        Property {
            name: "level",
            values: &[
                "0", "1", "2", "3", "4", "5", "6", "7", "8", "9", "10", "11", "12", "13", "14",
                "15",
            ],
        },
    ];
    const ITEM: Option<u16> = None;
    const WALL_VARIANT: Option<BlockId> = None;
    const SETTINGS: BlockSettings = BlockSettings {
        hardness: 100.0,
        resistance: 100.0,
        friction: 0.6,
        speed_factor: 1.0,
        jump_factor: 1.0,
        requires_correct_tool: false,
        ignited_by_lava: false,
    };
    const LIGHT_EMISSION: PerState<u8> = PerState::Uniform(0);
    const LIGHT_DAMPENING: PerState<u8> = PerState::Uniform(1);
    const FLAGS: PerState<StateFlags> = PerState::Uniform(StateFlags(192));
    type Behavior = crate::family::Plain;
}
