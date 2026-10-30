use crate::{
    Block, BlockId, BlockSettings, FaceSupport, PerState, Property, ShapeId, StateFlags, StateId,
};

pub struct PurpleWoolSlab;

impl Block for PurpleWoolSlab {
    const NAME: &'static str = "minecraft:purple_wool_slab";
    const ID: BlockId = BlockId(193);
    const FIRST_STATE: StateId = StateId(3765);
    const STATE_COUNT: u16 = 6;
    const DEFAULT_STATE: StateId = StateId(3768);
    const PROPERTIES: &'static [Property] = &[
        Property { name: "type", values: &["top", "bottom", "double"] },
        Property { name: "waterlogged", values: &["true", "false"] },
    ];
    const ITEM: Option<u16> = Some(292);
    const WALL_VARIANT: Option<BlockId> = None;
    const SETTINGS: BlockSettings = BlockSettings {
        hardness: 0.8,
        resistance: 0.8,
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
        1, 0, 1, 0, 15, 15,
    ]);
    const FLAGS: PerState<StateFlags> = PerState::Varies(&[
        StateFlags(2190), StateFlags(2094), StateFlags(2190), StateFlags(2094), StateFlags(3230),
        StateFlags(3102),
    ]);
    const COLLISION: PerState<ShapeId> = PerState::Varies(crate::shapes::COLLISION_WHITE_WOOL_SLAB);
    const FACE_SUPPORT: PerState<FaceSupport> =
        PerState::Varies(crate::shapes::SUPPORT_WHITE_WOOL_SLAB);
    type Behavior = crate::family::Slab;
}
