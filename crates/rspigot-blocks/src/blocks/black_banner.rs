use crate::{Block, BlockId, BlockSettings, PerState, Property, StateFlags, StateId};

pub struct BlackBanner;

impl Block for BlackBanner {
    const NAME: &'static str = "minecraft:black_banner";
    const ID: BlockId = BlockId(632);
    const FIRST_STATE: StateId = StateId(15014);
    const STATE_COUNT: u16 = 16;
    const DEFAULT_STATE: StateId = StateId(15022);
    const PROPERTIES: &'static [Property] = &[
        Property {
            name: "rotation",
            values: &[
                "0", "1", "2", "3", "4", "5", "6", "7", "8", "9", "10", "11", "12", "13", "14",
                "15",
            ],
        },
    ];
    const ITEM: Option<u16> = Some(1432);
    const WALL_VARIANT: Option<BlockId> = Some(BlockId(648));
    const SETTINGS: BlockSettings = BlockSettings {
        hardness: 1.0,
        resistance: 1.0,
        friction: 0.6,
        speed_factor: 1.0,
        jump_factor: 1.0,
        requires_correct_tool: false,
        ignited_by_lava: true,
    };
    const LIGHT_EMISSION: PerState<u8> = PerState::Uniform(0);
    const LIGHT_DAMPENING: PerState<u8> = PerState::Uniform(0);
    const FLAGS: PerState<StateFlags> = PerState::Uniform(StateFlags(34));
    type Behavior = crate::family::StandingWithWall;
}
