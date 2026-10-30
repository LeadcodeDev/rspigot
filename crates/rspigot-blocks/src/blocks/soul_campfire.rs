use crate::{
    Block, BlockId, BlockSettings, FaceSupport, PerState, Property, ShapeId, StateFlags, StateId,
};

pub struct SoulCampfire;

impl Block for SoulCampfire {
    const NAME: &'static str = "minecraft:soul_campfire";
    const ID: BlockId = BlockId(950);
    const FIRST_STATE: StateId = StateId(24266);
    const STATE_COUNT: u16 = 32;
    const DEFAULT_STATE: StateId = StateId(24269);
    const PROPERTIES: &'static [Property] = &[
        Property { name: "facing", values: &["north", "south", "west", "east"] },
        Property { name: "lit", values: &["true", "false"] },
        Property { name: "signal_fire", values: &["true", "false"] },
        Property { name: "waterlogged", values: &["true", "false"] },
    ];
    const ITEM: Option<u16> = Some(1528);
    const WALL_VARIANT: Option<BlockId> = None;
    const SETTINGS: BlockSettings = BlockSettings {
        hardness: 2.0,
        resistance: 2.0,
        friction: 0.6,
        speed_factor: 1.0,
        jump_factor: 1.0,
        requires_correct_tool: false,
        ignited_by_lava: true,
    };
    const LIGHT_EMISSION: PerState<u8> = PerState::Varies(&[
        10, 10, 10, 10, 0, 0, 0, 0, 10, 10, 10, 10, 0, 0, 0, 0, 10, 10, 10, 10, 0, 0, 0, 0, 10, 10,
        10, 10, 0, 0, 0, 0,
    ]);
    const LIGHT_DAMPENING: PerState<u8> = PerState::Varies(&[
        1, 0, 1, 0, 1, 0, 1, 0, 1, 0, 1, 0, 1, 0, 1, 0, 1, 0, 1, 0, 1, 0, 1, 0, 1, 0, 1, 0, 1, 0, 1,
        0,
    ]);
    const FLAGS: PerState<StateFlags> = PerState::Varies(&[
        StateFlags(134), StateFlags(38), StateFlags(134), StateFlags(38), StateFlags(134),
        StateFlags(38), StateFlags(134), StateFlags(38), StateFlags(134), StateFlags(38),
        StateFlags(134), StateFlags(38), StateFlags(134), StateFlags(38), StateFlags(134),
        StateFlags(38), StateFlags(134), StateFlags(38), StateFlags(134), StateFlags(38),
        StateFlags(134), StateFlags(38), StateFlags(134), StateFlags(38), StateFlags(134),
        StateFlags(38), StateFlags(134), StateFlags(38), StateFlags(134), StateFlags(38),
        StateFlags(134), StateFlags(38),
    ]);
    const COLLISION: PerState<ShapeId> = PerState::Uniform(ShapeId(297));
    const FACE_SUPPORT: PerState<FaceSupport> = PerState::Uniform(FaceSupport(1, 1, 1));
    type Behavior = crate::family::HorizontalToward;
}
