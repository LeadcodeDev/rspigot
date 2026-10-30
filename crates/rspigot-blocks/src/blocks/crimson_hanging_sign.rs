use crate::{Block, BlockId, BlockSettings, PerState, Property, StateFlags, StateId};

pub struct CrimsonHangingSign;

impl Block for CrimsonHangingSign {
    const NAME: &'static str = "minecraft:crimson_hanging_sign";
    const ID: BlockId = BlockId(288);
    const FIRST_STATE: StateId = StateId(8015);
    const STATE_COUNT: u16 = 64;
    const DEFAULT_STATE: StateId = StateId(8064);
    const PROPERTIES: &'static [Property] = &[
        Property { name: "attached", values: &["true", "false"] },
        Property {
            name: "rotation",
            values: &[
                "0", "1", "2", "3", "4", "5", "6", "7", "8", "9", "10", "11", "12", "13", "14",
                "15",
            ],
        },
        Property { name: "waterlogged", values: &["true", "false"] },
    ];
    const ITEM: Option<u16> = Some(1126);
    const WALL_VARIANT: Option<BlockId> = Some(BlockId(303));
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
    const LIGHT_DAMPENING: PerState<u8> = PerState::Varies(&[
        1, 0, 1, 0, 1, 0, 1, 0, 1, 0, 1, 0, 1, 0, 1, 0, 1, 0, 1, 0, 1, 0, 1, 0, 1, 0, 1, 0, 1, 0, 1,
        0, 1, 0, 1, 0, 1, 0, 1, 0, 1, 0, 1, 0, 1, 0, 1, 0, 1, 0, 1, 0, 1, 0, 1, 0, 1, 0, 1, 0, 1, 0,
        1, 0,
    ]);
    const FLAGS: PerState<StateFlags> = PerState::Varies(&[
        StateFlags(130), StateFlags(34), StateFlags(130), StateFlags(34), StateFlags(130),
        StateFlags(34), StateFlags(130), StateFlags(34), StateFlags(130), StateFlags(34),
        StateFlags(130), StateFlags(34), StateFlags(130), StateFlags(34), StateFlags(130),
        StateFlags(34), StateFlags(130), StateFlags(34), StateFlags(130), StateFlags(34),
        StateFlags(130), StateFlags(34), StateFlags(130), StateFlags(34), StateFlags(130),
        StateFlags(34), StateFlags(130), StateFlags(34), StateFlags(130), StateFlags(34),
        StateFlags(130), StateFlags(34), StateFlags(130), StateFlags(34), StateFlags(130),
        StateFlags(34), StateFlags(130), StateFlags(34), StateFlags(130), StateFlags(34),
        StateFlags(130), StateFlags(34), StateFlags(130), StateFlags(34), StateFlags(130),
        StateFlags(34), StateFlags(130), StateFlags(34), StateFlags(130), StateFlags(34),
        StateFlags(130), StateFlags(34), StateFlags(130), StateFlags(34), StateFlags(130),
        StateFlags(34), StateFlags(130), StateFlags(34), StateFlags(130), StateFlags(34),
        StateFlags(130), StateFlags(34), StateFlags(130), StateFlags(34),
    ]);
    type Behavior = crate::family::StandingWithWall;
}
