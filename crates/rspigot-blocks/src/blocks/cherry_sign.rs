use crate::{Block, BlockId, BlockSettings, PerState, Property, StateFlags, StateId};

pub struct CherrySign;

impl Block for CherrySign {
    const NAME: &'static str = "minecraft:cherry_sign";
    const ID: BlockId = BlockId(258);
    const FIRST_STATE: StateId = StateId(7019);
    const STATE_COUNT: u16 = 32;
    const DEFAULT_STATE: StateId = StateId(7036);
    const PROPERTIES: &'static [Property] = &[
        Property {
            name: "rotation",
            values: &[
                "0", "1", "2", "3", "4", "5", "6", "7", "8", "9", "10", "11", "12", "13", "14",
                "15",
            ],
        },
        Property { name: "waterlogged", values: &["true", "false"] },
    ];
    const ITEM: Option<u16> = Some(1107);
    const WALL_VARIANT: Option<BlockId> = Some(BlockId(273));
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
    const LIGHT_DAMPENING: PerState<u8> = PerState::Varies(&[
        1, 0, 1, 0, 1, 0, 1, 0, 1, 0, 1, 0, 1, 0, 1, 0, 1, 0, 1, 0, 1, 0, 1, 0, 1, 0, 1, 0, 1, 0, 1,
        0,
    ]);
    const FLAGS: PerState<StateFlags> = PerState::Varies(&[
        StateFlags(128), StateFlags(32), StateFlags(128), StateFlags(32), StateFlags(128),
        StateFlags(32), StateFlags(128), StateFlags(32), StateFlags(128), StateFlags(32),
        StateFlags(128), StateFlags(32), StateFlags(128), StateFlags(32), StateFlags(128),
        StateFlags(32), StateFlags(128), StateFlags(32), StateFlags(128), StateFlags(32),
        StateFlags(128), StateFlags(32), StateFlags(128), StateFlags(32), StateFlags(128),
        StateFlags(32), StateFlags(128), StateFlags(32), StateFlags(128), StateFlags(32),
        StateFlags(128), StateFlags(32),
    ]);
    type Behavior = crate::family::StandingWithWall;
}
