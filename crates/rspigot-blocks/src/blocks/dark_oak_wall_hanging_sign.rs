use crate::{
    Block, BlockId, BlockSettings, FaceSupport, PerState, Property, ShapeId, StateFlags, StateId,
};

pub struct DarkOakWallHangingSign;

impl Block for DarkOakWallHangingSign {
    const NAME: &'static str = "minecraft:dark_oak_wall_hanging_sign";
    const ID: BlockId = BlockId(299);
    const FIRST_STATE: StateId = StateId(8383);
    const STATE_COUNT: u16 = 8;
    const DEFAULT_STATE: StateId = StateId(8384);
    const PROPERTIES: &'static [Property] = &[
        Property { name: "facing", values: &["north", "south", "west", "east"] },
        Property { name: "waterlogged", values: &["true", "false"] },
    ];
    const ITEM: Option<u16> = Some(1121);
    const WALL_VARIANT: Option<BlockId> = None;
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
        1, 0, 1, 0, 1, 0, 1, 0,
    ]);
    const FLAGS: PerState<StateFlags> = PerState::Varies(&[
        StateFlags(134), StateFlags(38), StateFlags(134), StateFlags(38), StateFlags(134),
        StateFlags(38), StateFlags(134), StateFlags(38),
    ]);
    const COLLISION: PerState<ShapeId> =
        PerState::Varies(crate::shapes::COLLISION_OAK_WALL_HANGING_SIGN);
    const FACE_SUPPORT: PerState<FaceSupport> = PerState::Uniform(FaceSupport(0, 3, 0));
    type Behavior = crate::family::Plain;
}
