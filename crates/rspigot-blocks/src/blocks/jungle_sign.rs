use crate::{
    Block, BlockId, BlockSettings, FaceSupport, PerState, Property, ShapeId, StateFlags, StateId,
};

pub struct JungleSign;

impl Block for JungleSign {
    const NAME: &'static str = "minecraft:jungle_sign";
    const ID: BlockId = BlockId(259);
    const FIRST_STATE: StateId = StateId(7051);
    const STATE_COUNT: u16 = 32;
    const DEFAULT_STATE: StateId = StateId(7068);
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
    const ITEM: Option<u16> = Some(1105);
    const WALL_VARIANT: Option<BlockId> = Some(BlockId(274));
    const SETTINGS: BlockSettings = BlockSettings {
        hardness: 1.0,
        resistance: 1.0,
        friction: 0.6,
        speed_factor: 1.0,
        jump_factor: 1.0,
        bounce_restitution: 0.0,
        fall_distance_reduction: 0.0,
        requires_correct_tool: false,
        ignited_by_lava: true,
    };
    const LIGHT_EMISSION: PerState<u8> = PerState::Uniform(0);
    const LIGHT_DAMPENING: PerState<u8> = PerState::Varies(&[
        1, 0, 1, 0, 1, 0, 1, 0, 1, 0, 1, 0, 1, 0, 1, 0, 1, 0, 1, 0, 1, 0, 1, 0, 1, 0, 1, 0, 1, 0, 1,
        0,
    ]);
    const FLAGS: PerState<StateFlags> = PerState::Varies(&[
        StateFlags(2176), StateFlags(2080), StateFlags(2176), StateFlags(2080), StateFlags(2176),
        StateFlags(2080), StateFlags(2176), StateFlags(2080), StateFlags(2176), StateFlags(2080),
        StateFlags(2176), StateFlags(2080), StateFlags(2176), StateFlags(2080), StateFlags(2176),
        StateFlags(2080), StateFlags(2176), StateFlags(2080), StateFlags(2176), StateFlags(2080),
        StateFlags(2176), StateFlags(2080), StateFlags(2176), StateFlags(2080), StateFlags(2176),
        StateFlags(2080), StateFlags(2176), StateFlags(2080), StateFlags(2176), StateFlags(2080),
        StateFlags(2176), StateFlags(2080),
    ]);
    const COLLISION: PerState<ShapeId> = PerState::Uniform(ShapeId(0));
    const FACE_SUPPORT: PerState<FaceSupport> = PerState::Uniform(FaceSupport(0, 0, 0));
    type Behavior = crate::family::StandingWithWall;
}
