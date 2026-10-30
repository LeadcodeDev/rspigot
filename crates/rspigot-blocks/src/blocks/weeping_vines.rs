use crate::{Block, BlockId, BlockSettings, PerState, Property, StateFlags, StateId};

pub struct WeepingVines;

impl Block for WeepingVines {
    const NAME: &'static str = "minecraft:weeping_vines";
    const ID: BlockId = BlockId(968);
    const FIRST_STATE: StateId = StateId(24334);
    const STATE_COUNT: u16 = 26;
    const DEFAULT_STATE: StateId = StateId(24334);
    const PROPERTIES: &'static [Property] = &[
        Property {
            name: "age",
            values: &[
                "0", "1", "2", "3", "4", "5", "6", "7", "8", "9", "10", "11", "12", "13", "14",
                "15", "16", "17", "18", "19", "20", "21", "22", "23", "24", "25",
            ],
        },
    ];
    const ITEM: Option<u16> = Some(325);
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
    const FLAGS: PerState<StateFlags> = PerState::Varies(&[
        StateFlags(544), StateFlags(544), StateFlags(544), StateFlags(544), StateFlags(544),
        StateFlags(544), StateFlags(544), StateFlags(544), StateFlags(544), StateFlags(544),
        StateFlags(544), StateFlags(544), StateFlags(544), StateFlags(544), StateFlags(544),
        StateFlags(544), StateFlags(544), StateFlags(544), StateFlags(544), StateFlags(544),
        StateFlags(544), StateFlags(544), StateFlags(544), StateFlags(544), StateFlags(544),
        StateFlags(32),
    ]);
    type Behavior = crate::family::Plain;
}
