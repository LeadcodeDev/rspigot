use crate::{
    Block, BlockId, BlockSettings, FaceSupport, PerState, Property, ShapeId, StateFlags, StateId,
};

pub struct MangroveShelf;

impl Block for MangroveShelf {
    const NAME: &'static str = "minecraft:mangrove_shelf";
    const ID: BlockId = BlockId(230);
    const FIRST_STATE: StateId = StateId(4540);
    const STATE_COUNT: u16 = 64;
    const DEFAULT_STATE: StateId = StateId(4549);
    const PROPERTIES: &'static [Property] = &[
        Property { name: "facing", values: &["north", "south", "west", "east"] },
        Property { name: "powered", values: &["true", "false"] },
        Property { name: "side_chain", values: &["unconnected", "right", "center", "left"] },
        Property { name: "waterlogged", values: &["true", "false"] },
    ];
    const ITEM: Option<u16> = Some(384);
    const WALL_VARIANT: Option<BlockId> = None;
    const SETTINGS: BlockSettings = BlockSettings {
        hardness: 2.0,
        resistance: 3.0,
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
        StateFlags(2190), StateFlags(2094), StateFlags(2190), StateFlags(2094),
    ]);
    const COLLISION: PerState<ShapeId> = PerState::Varies(crate::shapes::COLLISION_ACACIA_SHELF);
    const FACE_SUPPORT: PerState<FaceSupport> =
        PerState::Varies(crate::shapes::SUPPORT_ACACIA_SHELF);
    type Behavior = crate::family::HorizontalAway;
}
