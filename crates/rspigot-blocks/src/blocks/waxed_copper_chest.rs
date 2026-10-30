use crate::{
    Block, BlockId, BlockSettings, FaceSupport, PerState, Property, ShapeId, StateFlags, StateId,
};

pub struct WaxedCopperChest;

impl Block for WaxedCopperChest {
    const NAME: &'static str = "minecraft:waxed_copper_chest";
    const ID: BlockId = BlockId(1202);
    const FIRST_STATE: StateId = StateId(33021);
    const STATE_COUNT: u16 = 24;
    const DEFAULT_STATE: StateId = StateId(33022);
    const PROPERTIES: &'static [Property] = &[
        Property { name: "facing", values: &["north", "south", "west", "east"] },
        Property { name: "type", values: &["single", "left", "right"] },
        Property { name: "waterlogged", values: &["true", "false"] },
    ];
    const ITEM: Option<u16> = Some(1641);
    const WALL_VARIANT: Option<BlockId> = None;
    const SETTINGS: BlockSettings = BlockSettings {
        hardness: 3.0,
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
        1, 0, 1, 0, 1, 0, 1, 0, 1, 0, 1, 0, 1, 0, 1, 0, 1, 0, 1, 0, 1, 0, 1, 0,
    ]);
    const FLAGS: PerState<StateFlags> = PerState::Varies(&[
        StateFlags(2190), StateFlags(2094), StateFlags(2190), StateFlags(2094), StateFlags(2190),
        StateFlags(2094), StateFlags(2190), StateFlags(2094), StateFlags(2190), StateFlags(2094),
        StateFlags(2190), StateFlags(2094), StateFlags(2190), StateFlags(2094), StateFlags(2190),
        StateFlags(2094), StateFlags(2190), StateFlags(2094), StateFlags(2190), StateFlags(2094),
        StateFlags(2190), StateFlags(2094), StateFlags(2190), StateFlags(2094),
    ]);
    const COLLISION: PerState<ShapeId> = PerState::Varies(crate::shapes::COLLISION_CHEST);
    const FACE_SUPPORT: PerState<FaceSupport> = PerState::Varies(crate::shapes::SUPPORT_CHEST);
    type Behavior = crate::family::HorizontalAway;
}
