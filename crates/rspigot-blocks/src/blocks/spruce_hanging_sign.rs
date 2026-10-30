use crate::{
    Block, BlockId, BlockSettings, FaceSupport, PerState, Property, ShapeId, StateFlags, StateId,
};

pub struct SpruceHangingSign;

impl Block for SpruceHangingSign {
    const NAME: &'static str = "minecraft:spruce_hanging_sign";
    const ID: BlockId = BlockId(281);
    const FIRST_STATE: StateId = StateId(7567);
    const STATE_COUNT: u16 = 64;
    const DEFAULT_STATE: StateId = StateId(7616);
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
    const ITEM: Option<u16> = Some(1116);
    const WALL_VARIANT: Option<BlockId> = Some(BlockId(294));
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
        0, 1, 0, 1, 0, 1, 0, 1, 0, 1, 0, 1, 0, 1, 0, 1, 0, 1, 0, 1, 0, 1, 0, 1, 0, 1, 0, 1, 0, 1, 0,
        1, 0,
    ]);
    const FLAGS: PerState<StateFlags> = PerState::Varies(&[
        StateFlags(18562), StateFlags(18466), StateFlags(18562), StateFlags(18466),
        StateFlags(18562), StateFlags(18466), StateFlags(18562), StateFlags(18466),
        StateFlags(18562), StateFlags(18466), StateFlags(18562), StateFlags(18466),
        StateFlags(18562), StateFlags(18466), StateFlags(18562), StateFlags(18466),
        StateFlags(18562), StateFlags(18466), StateFlags(18562), StateFlags(18466),
        StateFlags(18562), StateFlags(18466), StateFlags(18562), StateFlags(18466),
        StateFlags(18562), StateFlags(18466), StateFlags(18562), StateFlags(18466),
        StateFlags(18562), StateFlags(18466), StateFlags(18562), StateFlags(18466),
        StateFlags(18562), StateFlags(18466), StateFlags(18562), StateFlags(18466),
        StateFlags(18562), StateFlags(18466), StateFlags(18562), StateFlags(18466),
        StateFlags(18562), StateFlags(18466), StateFlags(18562), StateFlags(18466),
        StateFlags(18562), StateFlags(18466), StateFlags(18562), StateFlags(18466),
        StateFlags(18562), StateFlags(18466), StateFlags(18562), StateFlags(18466),
        StateFlags(18562), StateFlags(18466), StateFlags(18562), StateFlags(18466),
        StateFlags(18562), StateFlags(18466), StateFlags(18562), StateFlags(18466),
        StateFlags(18562), StateFlags(18466), StateFlags(18562), StateFlags(18466),
    ]);
    const COLLISION: PerState<ShapeId> = PerState::Uniform(ShapeId(0));
    const FACE_SUPPORT: PerState<FaceSupport> =
        PerState::Varies(crate::shapes::SUPPORT_OAK_HANGING_SIGN);
    type Behavior = crate::family::StandingWithWall;
}
