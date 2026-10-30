use crate::{
    Block, BlockId, BlockSettings, FaceSupport, PerState, Property, ShapeId, StateFlags, StateId,
};

pub struct SculkShrieker;

impl Block for SculkShrieker {
    const NAME: &'static str = "minecraft:sculk_shrieker";
    const ID: BlockId = BlockId(1123);
    const FIRST_STATE: StateId = StateId(31131);
    const STATE_COUNT: u16 = 8;
    const DEFAULT_STATE: StateId = StateId(31138);
    const PROPERTIES: &'static [Property] = &[
        Property { name: "can_summon", values: &["true", "false"] },
        Property { name: "shrieking", values: &["true", "false"] },
        Property { name: "waterlogged", values: &["true", "false"] },
    ];
    const ITEM: Option<u16> = Some(506);
    const WALL_VARIANT: Option<BlockId> = None;
    const SETTINGS: BlockSettings = BlockSettings {
        hardness: 3.0,
        resistance: 3.0,
        friction: 0.6,
        speed_factor: 1.0,
        jump_factor: 1.0,
        requires_correct_tool: false,
        ignited_by_lava: false,
    };
    const LIGHT_EMISSION: PerState<u8> = PerState::Uniform(0);
    const LIGHT_DAMPENING: PerState<u8> = PerState::Uniform(1);
    const FLAGS: PerState<StateFlags> = PerState::Varies(&[
        StateFlags(142), StateFlags(14), StateFlags(142), StateFlags(14), StateFlags(142),
        StateFlags(14), StateFlags(142), StateFlags(14),
    ]);
    const COLLISION: PerState<ShapeId> = PerState::Uniform(ShapeId(54));
    const FACE_SUPPORT: PerState<FaceSupport> = PerState::Uniform(FaceSupport(1, 1, 1));
    type Behavior = crate::family::Plain;
}
