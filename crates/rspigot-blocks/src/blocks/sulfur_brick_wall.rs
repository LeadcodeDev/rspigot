use crate::{
    Block, BlockId, BlockSettings, FaceSupport, PerState, Property, ShapeId, StateFlags, StateId,
};

pub struct SulfurBrickWall;

impl Block for SulfurBrickWall {
    const NAME: &'static str = "minecraft:sulfur_brick_wall";
    const ID: BlockId = BlockId(1100);
    const FIRST_STATE: StateId = StateId(28958);
    const STATE_COUNT: u16 = 324;
    const DEFAULT_STATE: StateId = StateId(28961);
    const PROPERTIES: &'static [Property] = &[
        Property { name: "east", values: &["none", "low", "tall"] },
        Property { name: "north", values: &["none", "low", "tall"] },
        Property { name: "south", values: &["none", "low", "tall"] },
        Property { name: "up", values: &["true", "false"] },
        Property { name: "waterlogged", values: &["true", "false"] },
        Property { name: "west", values: &["none", "low", "tall"] },
    ];
    const ITEM: Option<u16> = Some(38);
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
        1, 1, 1, 0, 0, 0, 1, 1, 1, 0, 0, 0, 1, 1, 1, 0, 0, 0, 1, 1, 1, 0, 0, 0, 1, 1, 1, 0, 0, 0, 1,
        1, 1, 0, 0, 0, 1, 1, 1, 0, 0, 0, 1, 1, 1, 0, 0, 0, 1, 1, 1, 0, 0, 0, 1, 1, 1, 0, 0, 0, 1, 1,
        1, 0, 0, 0, 1, 1, 1, 0, 0, 0, 1, 1, 1, 0, 0, 0, 1, 1, 1, 0, 0, 0, 1, 1, 1, 0, 0, 0, 1, 1, 1,
        0, 0, 0, 1, 1, 1, 0, 0, 0, 1, 1, 1, 0, 0, 0, 1, 1, 1, 0, 0, 0, 1, 1, 1, 0, 0, 0, 1, 1, 1, 0,
        0, 0, 1, 1, 1, 0, 0, 0, 1, 1, 1, 0, 0, 0, 1, 1, 1, 0, 0, 0, 1, 1, 1, 0, 0, 0, 1, 1, 1, 0, 0,
        0, 1, 1, 1, 0, 0, 0, 1, 1, 1, 0, 0, 0, 1, 1, 1, 0, 0, 0, 1, 1, 1, 0, 0, 0, 1, 1, 1, 0, 0, 0,
        1, 1, 1, 0, 0, 0, 1, 1, 1, 0, 0, 0, 1, 1, 1, 0, 0, 0, 1, 1, 1, 0, 0, 0, 1, 1, 1, 0, 0, 0, 1,
        1, 1, 0, 0, 0, 1, 1, 1, 0, 0, 0, 1, 1, 1, 0, 0, 0, 1, 1, 1, 0, 0, 0, 1, 1, 1, 0, 0, 0, 1, 1,
        1, 0, 0, 0, 1, 1, 1, 0, 0, 0, 1, 1, 1, 0, 0, 0, 1, 1, 1, 0, 0, 0, 1, 1, 1, 0, 0, 0, 1, 1, 1,
        0, 0, 0, 1, 1, 1, 0, 0, 0, 1, 1, 1, 0, 0, 0, 1, 1, 1, 0, 0, 0, 1, 1, 1, 0, 0, 0, 1, 1, 1, 0,
        0, 0, 1, 1, 1, 0, 0, 0, 1, 1, 1, 0, 0, 0,
    ]);
    const FLAGS: PerState<StateFlags> = PerState::Varies(&[
        StateFlags(2190), StateFlags(2190), StateFlags(2190), StateFlags(2094), StateFlags(2094),
        StateFlags(2094), StateFlags(2186), StateFlags(2190), StateFlags(2190), StateFlags(2090),
        StateFlags(2094), StateFlags(2094), StateFlags(2190), StateFlags(2190), StateFlags(2190),
        StateFlags(2094), StateFlags(2094), StateFlags(2094), StateFlags(2190), StateFlags(2190),
        StateFlags(2190), StateFlags(2094), StateFlags(2094), StateFlags(2094), StateFlags(2190),
        StateFlags(2190), StateFlags(2190), StateFlags(2094), StateFlags(2094), StateFlags(2094),
        StateFlags(2190), StateFlags(2190), StateFlags(2190), StateFlags(2094), StateFlags(2094),
        StateFlags(2094), StateFlags(2190), StateFlags(2190), StateFlags(2190), StateFlags(2094),
        StateFlags(2094), StateFlags(2094), StateFlags(2190), StateFlags(2190), StateFlags(2190),
        StateFlags(2094), StateFlags(2094), StateFlags(2094), StateFlags(2190), StateFlags(2190),
        StateFlags(2190), StateFlags(2094), StateFlags(2094), StateFlags(2094), StateFlags(2190),
        StateFlags(2190), StateFlags(2190), StateFlags(2094), StateFlags(2094), StateFlags(2094),
        StateFlags(2190), StateFlags(2190), StateFlags(2190), StateFlags(2094), StateFlags(2094),
        StateFlags(2094), StateFlags(2190), StateFlags(2190), StateFlags(2190), StateFlags(2094),
        StateFlags(2094), StateFlags(2094), StateFlags(2190), StateFlags(2190), StateFlags(2190),
        StateFlags(2094), StateFlags(2094), StateFlags(2094), StateFlags(2190), StateFlags(2190),
        StateFlags(2190), StateFlags(2094), StateFlags(2094), StateFlags(2094), StateFlags(2190),
        StateFlags(2190), StateFlags(2190), StateFlags(2094), StateFlags(2094), StateFlags(2094),
        StateFlags(2190), StateFlags(2190), StateFlags(2190), StateFlags(2094), StateFlags(2094),
        StateFlags(2094), StateFlags(2190), StateFlags(2190), StateFlags(2190), StateFlags(2094),
        StateFlags(2094), StateFlags(2094), StateFlags(2190), StateFlags(2190), StateFlags(2190),
        StateFlags(2094), StateFlags(2094), StateFlags(2094), StateFlags(2190), StateFlags(2190),
        StateFlags(2190), StateFlags(2094), StateFlags(2094), StateFlags(2094), StateFlags(2190),
        StateFlags(2190), StateFlags(2190), StateFlags(2094), StateFlags(2094), StateFlags(2094),
        StateFlags(2190), StateFlags(2190), StateFlags(2190), StateFlags(2094), StateFlags(2094),
        StateFlags(2094), StateFlags(2190), StateFlags(2190), StateFlags(2190), StateFlags(2094),
        StateFlags(2094), StateFlags(2094), StateFlags(2190), StateFlags(2190), StateFlags(2190),
        StateFlags(2094), StateFlags(2094), StateFlags(2094), StateFlags(2190), StateFlags(2190),
        StateFlags(2190), StateFlags(2094), StateFlags(2094), StateFlags(2094), StateFlags(2190),
        StateFlags(2190), StateFlags(2190), StateFlags(2094), StateFlags(2094), StateFlags(2094),
        StateFlags(2190), StateFlags(2190), StateFlags(2190), StateFlags(2094), StateFlags(2094),
        StateFlags(2094), StateFlags(2190), StateFlags(2190), StateFlags(2190), StateFlags(2094),
        StateFlags(2094), StateFlags(2094), StateFlags(2190), StateFlags(2190), StateFlags(2190),
        StateFlags(2094), StateFlags(2094), StateFlags(2094), StateFlags(2190), StateFlags(2190),
        StateFlags(2190), StateFlags(2094), StateFlags(2094), StateFlags(2094), StateFlags(2190),
        StateFlags(2190), StateFlags(2190), StateFlags(2094), StateFlags(2094), StateFlags(2094),
        StateFlags(2190), StateFlags(2190), StateFlags(2190), StateFlags(2094), StateFlags(2094),
        StateFlags(2094), StateFlags(2190), StateFlags(2190), StateFlags(2190), StateFlags(2094),
        StateFlags(2094), StateFlags(2094), StateFlags(2190), StateFlags(2190), StateFlags(2190),
        StateFlags(2094), StateFlags(2094), StateFlags(2094), StateFlags(2190), StateFlags(2190),
        StateFlags(2190), StateFlags(2094), StateFlags(2094), StateFlags(2094), StateFlags(2190),
        StateFlags(2190), StateFlags(2190), StateFlags(2094), StateFlags(2094), StateFlags(2094),
        StateFlags(2190), StateFlags(2190), StateFlags(2190), StateFlags(2094), StateFlags(2094),
        StateFlags(2094), StateFlags(2190), StateFlags(2190), StateFlags(2190), StateFlags(2094),
        StateFlags(2094), StateFlags(2094), StateFlags(2190), StateFlags(2190), StateFlags(2190),
        StateFlags(2094), StateFlags(2094), StateFlags(2094), StateFlags(2190), StateFlags(2190),
        StateFlags(2190), StateFlags(2094), StateFlags(2094), StateFlags(2094), StateFlags(2190),
        StateFlags(2190), StateFlags(2190), StateFlags(2094), StateFlags(2094), StateFlags(2094),
        StateFlags(2190), StateFlags(2190), StateFlags(2190), StateFlags(2094), StateFlags(2094),
        StateFlags(2094), StateFlags(2190), StateFlags(2190), StateFlags(2190), StateFlags(2094),
        StateFlags(2094), StateFlags(2094), StateFlags(2190), StateFlags(2190), StateFlags(2190),
        StateFlags(2094), StateFlags(2094), StateFlags(2094), StateFlags(2190), StateFlags(2190),
        StateFlags(2190), StateFlags(2094), StateFlags(2094), StateFlags(2094), StateFlags(2190),
        StateFlags(2190), StateFlags(2190), StateFlags(2094), StateFlags(2094), StateFlags(2094),
        StateFlags(2190), StateFlags(2190), StateFlags(2190), StateFlags(2094), StateFlags(2094),
        StateFlags(2094), StateFlags(2190), StateFlags(2190), StateFlags(2190), StateFlags(2094),
        StateFlags(2094), StateFlags(2094), StateFlags(2190), StateFlags(2190), StateFlags(2190),
        StateFlags(2094), StateFlags(2094), StateFlags(2094), StateFlags(2190), StateFlags(2190),
        StateFlags(2190), StateFlags(2094), StateFlags(2094), StateFlags(2094), StateFlags(2190),
        StateFlags(2190), StateFlags(2190), StateFlags(2094), StateFlags(2094), StateFlags(2094),
        StateFlags(2190), StateFlags(2190), StateFlags(2190), StateFlags(2094), StateFlags(2094),
        StateFlags(2094), StateFlags(2190), StateFlags(2190), StateFlags(2190), StateFlags(2094),
        StateFlags(2094), StateFlags(2094), StateFlags(2190), StateFlags(2190), StateFlags(2190),
        StateFlags(2094), StateFlags(2094), StateFlags(2094), StateFlags(2190), StateFlags(2190),
        StateFlags(2190), StateFlags(2094), StateFlags(2094), StateFlags(2094),
    ]);
    const COLLISION: PerState<ShapeId> =
        PerState::Varies(crate::shapes::COLLISION_RESIN_BRICK_WALL);
    const FACE_SUPPORT: PerState<FaceSupport> =
        PerState::Varies(crate::shapes::SUPPORT_RESIN_BRICK_WALL);
    type Behavior = crate::family::Plain;
}
