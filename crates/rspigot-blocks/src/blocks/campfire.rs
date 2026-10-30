use crate::{
    Block, BlockId, BlockSettings, FaceSupport, PerState, Property, ShapeId, StateFlags, StateId,
};

pub struct Campfire;

impl Block for Campfire {
    const NAME: &'static str = "minecraft:campfire";
    const ID: BlockId = BlockId(949);
    const FIRST_STATE: StateId = StateId(24234);
    const STATE_COUNT: u16 = 32;
    const DEFAULT_STATE: StateId = StateId(24237);
    const PROPERTIES: &'static [Property] = &[
        Property { name: "facing", values: &["north", "south", "west", "east"] },
        Property { name: "lit", values: &["true", "false"] },
        Property { name: "signal_fire", values: &["true", "false"] },
        Property { name: "waterlogged", values: &["true", "false"] },
    ];
    const ITEM: Option<u16> = Some(1527);
    const WALL_VARIANT: Option<BlockId> = None;
    const SETTINGS: BlockSettings = BlockSettings {
        hardness: 2.0,
        resistance: 2.0,
        friction: 0.6,
        speed_factor: 1.0,
        jump_factor: 1.0,
        bounce_restitution: 0.0,
        fall_distance_reduction: 0.0,
        requires_correct_tool: false,
        ignited_by_lava: true,
    };
    const LIGHT_EMISSION: PerState<u8> = PerState::Varies(&[
        15, 15, 15, 15, 0, 0, 0, 0, 15, 15, 15, 15, 0, 0, 0, 0, 15, 15, 15, 15, 0, 0, 0, 0, 15, 15,
        15, 15, 0, 0, 0, 0,
    ]);
    const LIGHT_DAMPENING: PerState<u8> = PerState::Varies(&[
        1, 0, 1, 0, 1, 0, 1, 0, 1, 0, 1, 0, 1, 0, 1, 0, 1, 0, 1, 0, 1, 0, 1, 0, 1, 0, 1, 0, 1, 0, 1,
        0,
    ]);
    const FLAGS: PerState<StateFlags> = PerState::Varies(&[
        StateFlags(2182), StateFlags(2086), StateFlags(2182), StateFlags(2086), StateFlags(2182),
        StateFlags(2086), StateFlags(2182), StateFlags(2086), StateFlags(2182), StateFlags(2086),
        StateFlags(2182), StateFlags(2086), StateFlags(2182), StateFlags(2086), StateFlags(2182),
        StateFlags(2086), StateFlags(2182), StateFlags(2086), StateFlags(2182), StateFlags(2086),
        StateFlags(2182), StateFlags(2086), StateFlags(2182), StateFlags(2086), StateFlags(2182),
        StateFlags(2086), StateFlags(2182), StateFlags(2086), StateFlags(2182), StateFlags(2086),
        StateFlags(2182), StateFlags(2086),
    ]);
    const COLLISION: PerState<ShapeId> = PerState::Uniform(ShapeId(297));
    const FACE_SUPPORT: PerState<FaceSupport> = PerState::Uniform(FaceSupport(1, 1, 1));
    type Behavior = crate::family::HorizontalToward;
}
