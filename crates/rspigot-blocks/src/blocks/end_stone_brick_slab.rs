use crate::{
    Block, BlockId, BlockSettings, FaceSupport, PerState, Property, ShapeId, StateFlags, StateId,
};

pub struct EndStoneBrickSlab;

impl Block for EndStoneBrickSlab {
    const NAME: &'static str = "minecraft:end_stone_brick_slab";
    const ID: BlockId = BlockId(906);
    const FIRST_STATE: StateId = StateId(19803);
    const STATE_COUNT: u16 = 6;
    const DEFAULT_STATE: StateId = StateId(19806);
    const PROPERTIES: &'static [Property] = &[
        Property { name: "type", values: &["top", "bottom", "double"] },
        Property { name: "waterlogged", values: &["true", "false"] },
    ];
    const ITEM: Option<u16> = Some(811);
    const WALL_VARIANT: Option<BlockId> = None;
    const SETTINGS: BlockSettings = BlockSettings {
        hardness: 3.0,
        resistance: 9.0,
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
        1, 0, 1, 0, 15, 15,
    ]);
    const FLAGS: PerState<StateFlags> = PerState::Varies(&[
        StateFlags(142), StateFlags(46), StateFlags(142), StateFlags(46), StateFlags(1182),
        StateFlags(1054),
    ]);
    const COLLISION: PerState<ShapeId> = PerState::Varies(crate::shapes::COLLISION_WHITE_WOOL_SLAB);
    const FACE_SUPPORT: PerState<FaceSupport> =
        PerState::Varies(crate::shapes::SUPPORT_WHITE_WOOL_SLAB);
    type Behavior = crate::family::Slab;
}
