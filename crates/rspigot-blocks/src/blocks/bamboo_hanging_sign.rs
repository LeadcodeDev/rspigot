use crate::{
    Block, BlockId, BlockSettings, FaceSupport, PerState, Property, ShapeId, StateFlags, StateId,
};

pub struct BambooHangingSign;

impl Block for BambooHangingSign {
    const NAME: &'static str = "minecraft:bamboo_hanging_sign";
    const ID: BlockId = BlockId(292);
    const FIRST_STATE: StateId = StateId(8271);
    const STATE_COUNT: u16 = 64;
    const DEFAULT_STATE: StateId = StateId(8320);
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
    const ITEM: Option<u16> = Some(1125);
    const WALL_VARIANT: Option<BlockId> = Some(BlockId(305));
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
    const COLLISION: PerState<ShapeId> = PerState::Uniform(ShapeId(0));
    const FACE_SUPPORT: PerState<FaceSupport> =
        PerState::Varies(crate::shapes::SUPPORT_OAK_HANGING_SIGN);
    type Behavior = crate::family::StandingWithWall;
}
