use crate::{
    Block, BlockId, BlockSettings, FaceSupport, PerState, Property, ShapeId, StateFlags, StateId,
};

pub struct Light;

impl Block for Light {
    const NAME: &'static str = "minecraft:light";
    const ID: BlockId = BlockId(579);
    const FIRST_STATE: StateId = StateId(14382);
    const STATE_COUNT: u16 = 32;
    const DEFAULT_STATE: StateId = StateId(14413);
    const PROPERTIES: &'static [Property] = &[
        Property {
            name: "level",
            values: &[
                "0", "1", "2", "3", "4", "5", "6", "7", "8", "9", "10", "11", "12", "13", "14",
                "15",
            ],
        },
        Property { name: "waterlogged", values: &["true", "false"] },
    ];
    const ITEM: Option<u16> = Some(578);
    const WALL_VARIANT: Option<BlockId> = None;
    const SETTINGS: BlockSettings = BlockSettings {
        hardness: -1.0,
        resistance: 3600000.8,
        friction: 0.6,
        speed_factor: 1.0,
        jump_factor: 1.0,
        requires_correct_tool: false,
        ignited_by_lava: false,
    };
    const LIGHT_EMISSION: PerState<u8> = PerState::Varies(&[
        0, 0, 1, 1, 2, 2, 3, 3, 4, 4, 5, 5, 6, 6, 7, 7, 8, 8, 9, 9, 10, 10, 11, 11, 12, 12, 13, 13,
        14, 14, 15, 15,
    ]);
    const LIGHT_DAMPENING: PerState<u8> = PerState::Varies(&[
        1, 0, 1, 0, 1, 0, 1, 0, 1, 0, 1, 0, 1, 0, 1, 0, 1, 0, 1, 0, 1, 0, 1, 0, 1, 0, 1, 0, 1, 0, 1,
        0,
    ]);
    const FLAGS: PerState<StateFlags> = PerState::Varies(&[
        StateFlags(192), StateFlags(96), StateFlags(192), StateFlags(96), StateFlags(192),
        StateFlags(96), StateFlags(192), StateFlags(96), StateFlags(192), StateFlags(96),
        StateFlags(192), StateFlags(96), StateFlags(192), StateFlags(96), StateFlags(192),
        StateFlags(96), StateFlags(192), StateFlags(96), StateFlags(192), StateFlags(96),
        StateFlags(192), StateFlags(96), StateFlags(192), StateFlags(96), StateFlags(192),
        StateFlags(96), StateFlags(192), StateFlags(96), StateFlags(192), StateFlags(96),
        StateFlags(192), StateFlags(96),
    ]);
    const COLLISION: PerState<ShapeId> = PerState::Uniform(ShapeId(0));
    const FACE_SUPPORT: PerState<FaceSupport> = PerState::Uniform(FaceSupport(0, 0, 0));
    type Behavior = crate::family::Plain;
}
