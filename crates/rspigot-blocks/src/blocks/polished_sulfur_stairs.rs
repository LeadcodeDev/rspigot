use crate::{
    Block, BlockId, BlockSettings, FaceSupport, PerState, Property, ShapeId, StateFlags, StateId,
};

pub struct PolishedSulfurStairs;

impl Block for PolishedSulfurStairs {
    const NAME: &'static str = "minecraft:polished_sulfur_stairs";
    const ID: BlockId = BlockId(1095);
    const FIRST_STATE: StateId = StateId(28467);
    const STATE_COUNT: u16 = 80;
    const DEFAULT_STATE: StateId = StateId(28478);
    const PROPERTIES: &'static [Property] = &[
        Property { name: "facing", values: &["north", "south", "west", "east"] },
        Property { name: "half", values: &["top", "bottom"] },
        Property {
            name: "shape",
            values: &[
                "straight", "inner_left", "inner_right", "outer_left", "outer_right",
            ],
        },
        Property { name: "waterlogged", values: &["true", "false"] },
    ];
    const ITEM: Option<u16> = Some(33);
    const WALL_VARIANT: Option<BlockId> = None;
    const SETTINGS: BlockSettings = BlockSettings {
        hardness: 1.5,
        resistance: 6.0,
        friction: 0.6,
        speed_factor: 1.0,
        jump_factor: 1.0,
        bounce_restitution: 0.0,
        fall_distance_reduction: 0.0,
        requires_correct_tool: true,
        ignited_by_lava: false,
    };
    const LIGHT_EMISSION: PerState<u8> = PerState::Uniform(0);
    const LIGHT_DAMPENING: PerState<u8> = PerState::Varies(&[
        1, 0, 1, 0, 1, 0, 1, 0, 1, 0, 1, 0, 1, 0, 1, 0, 1, 0, 1, 0, 1, 0, 1, 0, 1, 0, 1, 0, 1, 0, 1,
        0, 1, 0, 1, 0, 1, 0, 1, 0, 1, 0, 1, 0, 1, 0, 1, 0, 1, 0, 1, 0, 1, 0, 1, 0, 1, 0, 1, 0, 1, 0,
        1, 0, 1, 0, 1, 0, 1, 0, 1, 0, 1, 0, 1, 0, 1, 0, 1, 0,
    ]);
    const FLAGS: PerState<StateFlags> = PerState::Varies(&[
        StateFlags(2190), StateFlags(2094), StateFlags(2190), StateFlags(2094), StateFlags(2190),
        StateFlags(2094), StateFlags(2190), StateFlags(2094), StateFlags(2190), StateFlags(2094),
        StateFlags(2190), StateFlags(2094), StateFlags(2190), StateFlags(2094), StateFlags(2190),
        StateFlags(2094), StateFlags(2190), StateFlags(2094), StateFlags(2190), StateFlags(2094),
        StateFlags(2190), StateFlags(2094), StateFlags(2190), StateFlags(2094), StateFlags(2190),
        StateFlags(2094), StateFlags(2190), StateFlags(2094), StateFlags(2190), StateFlags(2094),
        StateFlags(2190), StateFlags(2094), StateFlags(2190), StateFlags(2094), StateFlags(2190),
        StateFlags(2094), StateFlags(2190), StateFlags(2094), StateFlags(2190), StateFlags(2094),
        StateFlags(2190), StateFlags(2094), StateFlags(2190), StateFlags(2094), StateFlags(2190),
        StateFlags(2094), StateFlags(2190), StateFlags(2094), StateFlags(2190), StateFlags(2094),
        StateFlags(2190), StateFlags(2094), StateFlags(2190), StateFlags(2094), StateFlags(2190),
        StateFlags(2094), StateFlags(2190), StateFlags(2094), StateFlags(2190), StateFlags(2094),
        StateFlags(2190), StateFlags(2094), StateFlags(2190), StateFlags(2094), StateFlags(2190),
        StateFlags(2094), StateFlags(2190), StateFlags(2094), StateFlags(2190), StateFlags(2094),
        StateFlags(2190), StateFlags(2094), StateFlags(2190), StateFlags(2094), StateFlags(2190),
        StateFlags(2094), StateFlags(2190), StateFlags(2094), StateFlags(2190), StateFlags(2094),
    ]);
    const COLLISION: PerState<ShapeId> =
        PerState::Varies(crate::shapes::COLLISION_WHITE_WOOL_STAIRS);
    const FACE_SUPPORT: PerState<FaceSupport> =
        PerState::Varies(crate::shapes::SUPPORT_WHITE_WOOL_STAIRS);
    type Behavior = crate::family::Stairs;
}
