use crate::{Block, BlockId, BlockSettings, PerState, Property, StateFlags, StateId};

pub struct PiglinHead;

impl Block for PiglinHead {
    const NAME: &'static str = "minecraft:piglin_head";
    const ID: BlockId = BlockId(518);
    const FIRST_STATE: StateId = StateId(12922);
    const STATE_COUNT: u16 = 32;
    const DEFAULT_STATE: StateId = StateId(12938);
    const PROPERTIES: &'static [Property] = &[
        Property { name: "powered", values: &["true", "false"] },
        Property {
            name: "rotation",
            values: &[
                "0", "1", "2", "3", "4", "5", "6", "7", "8", "9", "10", "11", "12", "13", "14",
                "15",
            ],
        },
    ];
    const ITEM: Option<u16> = Some(1390);
    const WALL_VARIANT: Option<BlockId> = Some(BlockId(519));
    const SETTINGS: BlockSettings = BlockSettings {
        hardness: 1.0,
        resistance: 1.0,
        friction: 0.6,
        speed_factor: 1.0,
        jump_factor: 1.0,
        requires_correct_tool: false,
        ignited_by_lava: false,
    };
    const LIGHT_EMISSION: PerState<u8> = PerState::Uniform(0);
    const LIGHT_DAMPENING: PerState<u8> = PerState::Uniform(0);
    const FLAGS: PerState<StateFlags> = PerState::Uniform(StateFlags(36));
    type Behavior = crate::family::StandingWithWall;
}
