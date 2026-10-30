use crate::{
    Block, BlockId, BlockSettings, FaceSupport, PerState, Property, ShapeId, StateFlags, StateId,
};

pub struct SculkVein;

impl Block for SculkVein {
    const NAME: &'static str = "minecraft:sculk_vein";
    const ID: BlockId = BlockId(1121);
    const FIRST_STATE: StateId = StateId(31001);
    const STATE_COUNT: u16 = 128;
    const DEFAULT_STATE: StateId = StateId(31128);
    const PROPERTIES: &'static [Property] = &[
        Property { name: "down", values: &["true", "false"] },
        Property { name: "east", values: &["true", "false"] },
        Property { name: "north", values: &["true", "false"] },
        Property { name: "south", values: &["true", "false"] },
        Property { name: "up", values: &["true", "false"] },
        Property { name: "waterlogged", values: &["true", "false"] },
        Property { name: "west", values: &["true", "false"] },
    ];
    const ITEM: Option<u16> = Some(504);
    const WALL_VARIANT: Option<BlockId> = None;
    const SETTINGS: BlockSettings = BlockSettings {
        hardness: 0.2,
        resistance: 0.2,
        friction: 0.6,
        speed_factor: 1.0,
        jump_factor: 1.0,
        bounce_restitution: 0.0,
        fall_distance_reduction: 0.0,
        requires_correct_tool: false,
        ignited_by_lava: false,
    };
    const LIGHT_EMISSION: PerState<u8> = PerState::Uniform(0);
    const LIGHT_DAMPENING: PerState<u8> = PerState::Varies(&[
        1, 1, 0, 0, 1, 1, 0, 0, 1, 1, 0, 0, 1, 1, 0, 0, 1, 1, 0, 0, 1, 1, 0, 0, 1, 1, 0, 0, 1, 1, 0,
        0, 1, 1, 0, 0, 1, 1, 0, 0, 1, 1, 0, 0, 1, 1, 0, 0, 1, 1, 0, 0, 1, 1, 0, 0, 1, 1, 0, 0, 1, 1,
        0, 0, 1, 1, 0, 0, 1, 1, 0, 0, 1, 1, 0, 0, 1, 1, 0, 0, 1, 1, 0, 0, 1, 1, 0, 0, 1, 1, 0, 0, 1,
        1, 0, 0, 1, 1, 0, 0, 1, 1, 0, 0, 1, 1, 0, 0, 1, 1, 0, 0, 1, 1, 0, 0, 1, 1, 0, 0, 1, 1, 0, 0,
        1, 1, 0, 1,
    ]);
    const FLAGS: PerState<StateFlags> = PerState::Varies(&[
        StateFlags(130), StateFlags(130), StateFlags(34), StateFlags(34), StateFlags(130),
        StateFlags(130), StateFlags(34), StateFlags(34), StateFlags(130), StateFlags(130),
        StateFlags(34), StateFlags(34), StateFlags(130), StateFlags(130), StateFlags(34),
        StateFlags(34), StateFlags(130), StateFlags(130), StateFlags(34), StateFlags(34),
        StateFlags(130), StateFlags(130), StateFlags(34), StateFlags(34), StateFlags(130),
        StateFlags(130), StateFlags(34), StateFlags(34), StateFlags(130), StateFlags(130),
        StateFlags(34), StateFlags(34), StateFlags(130), StateFlags(130), StateFlags(34),
        StateFlags(34), StateFlags(130), StateFlags(130), StateFlags(34), StateFlags(34),
        StateFlags(130), StateFlags(130), StateFlags(34), StateFlags(34), StateFlags(130),
        StateFlags(130), StateFlags(34), StateFlags(34), StateFlags(130), StateFlags(130),
        StateFlags(34), StateFlags(34), StateFlags(130), StateFlags(130), StateFlags(34),
        StateFlags(34), StateFlags(130), StateFlags(130), StateFlags(34), StateFlags(34),
        StateFlags(130), StateFlags(130), StateFlags(34), StateFlags(34), StateFlags(130),
        StateFlags(130), StateFlags(34), StateFlags(34), StateFlags(130), StateFlags(130),
        StateFlags(34), StateFlags(34), StateFlags(130), StateFlags(130), StateFlags(34),
        StateFlags(34), StateFlags(130), StateFlags(130), StateFlags(34), StateFlags(34),
        StateFlags(130), StateFlags(130), StateFlags(34), StateFlags(34), StateFlags(130),
        StateFlags(130), StateFlags(34), StateFlags(34), StateFlags(130), StateFlags(130),
        StateFlags(34), StateFlags(34), StateFlags(130), StateFlags(130), StateFlags(34),
        StateFlags(34), StateFlags(130), StateFlags(130), StateFlags(34), StateFlags(34),
        StateFlags(130), StateFlags(130), StateFlags(34), StateFlags(34), StateFlags(130),
        StateFlags(130), StateFlags(34), StateFlags(34), StateFlags(130), StateFlags(130),
        StateFlags(34), StateFlags(34), StateFlags(130), StateFlags(130), StateFlags(34),
        StateFlags(34), StateFlags(130), StateFlags(130), StateFlags(34), StateFlags(34),
        StateFlags(130), StateFlags(130), StateFlags(34), StateFlags(34), StateFlags(130),
        StateFlags(130), StateFlags(34), StateFlags(2),
    ]);
    const COLLISION: PerState<ShapeId> = PerState::Uniform(ShapeId(0));
    const FACE_SUPPORT: PerState<FaceSupport> = PerState::Uniform(FaceSupport(0, 0, 0));
    type Behavior = crate::family::Plain;
}
