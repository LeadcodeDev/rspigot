use crate::{
    Block, BlockId, BlockSettings, FaceSupport, PerState, Property, ShapeId, StateFlags, StateId,
};

pub struct GlowLichen;

impl Block for GlowLichen {
    const NAME: &'static str = "minecraft:glow_lichen";
    const ID: BlockId = BlockId(417);
    const FIRST_STATE: StateId = StateId(10124);
    const STATE_COUNT: u16 = 128;
    const DEFAULT_STATE: StateId = StateId(10251);
    const PROPERTIES: &'static [Property] = &[
        Property { name: "down", values: &["true", "false"] },
        Property { name: "east", values: &["true", "false"] },
        Property { name: "north", values: &["true", "false"] },
        Property { name: "south", values: &["true", "false"] },
        Property { name: "up", values: &["true", "false"] },
        Property { name: "waterlogged", values: &["true", "false"] },
        Property { name: "west", values: &["true", "false"] },
    ];
    const ITEM: Option<u16> = Some(485);
    const WALL_VARIANT: Option<BlockId> = None;
    const SETTINGS: BlockSettings = BlockSettings {
        hardness: 0.2,
        resistance: 0.2,
        friction: 0.6,
        speed_factor: 1.0,
        jump_factor: 1.0,
        requires_correct_tool: false,
        ignited_by_lava: true,
    };
    const LIGHT_EMISSION: PerState<u8> = PerState::Varies(&[
        7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7,
        7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7,
        7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7,
        7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7,
        7, 0, 7, 0,
    ]);
    const LIGHT_DAMPENING: PerState<u8> = PerState::Varies(&[
        1, 1, 0, 0, 1, 1, 0, 0, 1, 1, 0, 0, 1, 1, 0, 0, 1, 1, 0, 0, 1, 1, 0, 0, 1, 1, 0, 0, 1, 1, 0,
        0, 1, 1, 0, 0, 1, 1, 0, 0, 1, 1, 0, 0, 1, 1, 0, 0, 1, 1, 0, 0, 1, 1, 0, 0, 1, 1, 0, 0, 1, 1,
        0, 0, 1, 1, 0, 0, 1, 1, 0, 0, 1, 1, 0, 0, 1, 1, 0, 0, 1, 1, 0, 0, 1, 1, 0, 0, 1, 1, 0, 0, 1,
        1, 0, 0, 1, 1, 0, 0, 1, 1, 0, 0, 1, 1, 0, 0, 1, 1, 0, 0, 1, 1, 0, 0, 1, 1, 0, 0, 1, 1, 0, 0,
        1, 1, 0, 0,
    ]);
    const FLAGS: PerState<StateFlags> = PerState::Varies(&[
        StateFlags(192), StateFlags(192), StateFlags(96), StateFlags(96), StateFlags(192),
        StateFlags(192), StateFlags(96), StateFlags(96), StateFlags(192), StateFlags(192),
        StateFlags(96), StateFlags(96), StateFlags(192), StateFlags(192), StateFlags(96),
        StateFlags(96), StateFlags(192), StateFlags(192), StateFlags(96), StateFlags(96),
        StateFlags(192), StateFlags(192), StateFlags(96), StateFlags(96), StateFlags(192),
        StateFlags(192), StateFlags(96), StateFlags(96), StateFlags(192), StateFlags(192),
        StateFlags(96), StateFlags(96), StateFlags(192), StateFlags(192), StateFlags(96),
        StateFlags(96), StateFlags(192), StateFlags(192), StateFlags(96), StateFlags(96),
        StateFlags(192), StateFlags(192), StateFlags(96), StateFlags(96), StateFlags(192),
        StateFlags(192), StateFlags(96), StateFlags(96), StateFlags(192), StateFlags(192),
        StateFlags(96), StateFlags(96), StateFlags(192), StateFlags(192), StateFlags(96),
        StateFlags(96), StateFlags(192), StateFlags(192), StateFlags(96), StateFlags(96),
        StateFlags(192), StateFlags(192), StateFlags(96), StateFlags(96), StateFlags(192),
        StateFlags(192), StateFlags(96), StateFlags(96), StateFlags(192), StateFlags(192),
        StateFlags(96), StateFlags(96), StateFlags(192), StateFlags(192), StateFlags(96),
        StateFlags(96), StateFlags(192), StateFlags(192), StateFlags(96), StateFlags(96),
        StateFlags(192), StateFlags(192), StateFlags(96), StateFlags(96), StateFlags(192),
        StateFlags(192), StateFlags(96), StateFlags(96), StateFlags(192), StateFlags(192),
        StateFlags(96), StateFlags(96), StateFlags(192), StateFlags(192), StateFlags(96),
        StateFlags(96), StateFlags(192), StateFlags(192), StateFlags(96), StateFlags(96),
        StateFlags(192), StateFlags(192), StateFlags(96), StateFlags(96), StateFlags(192),
        StateFlags(192), StateFlags(96), StateFlags(96), StateFlags(192), StateFlags(192),
        StateFlags(96), StateFlags(96), StateFlags(192), StateFlags(192), StateFlags(96),
        StateFlags(96), StateFlags(192), StateFlags(192), StateFlags(96), StateFlags(96),
        StateFlags(192), StateFlags(192), StateFlags(96), StateFlags(96), StateFlags(192),
        StateFlags(192), StateFlags(96), StateFlags(96),
    ]);
    const COLLISION: PerState<ShapeId> = PerState::Uniform(ShapeId(0));
    const FACE_SUPPORT: PerState<FaceSupport> = PerState::Uniform(FaceSupport(0, 0, 0));
    type Behavior = crate::family::Plain;
}
