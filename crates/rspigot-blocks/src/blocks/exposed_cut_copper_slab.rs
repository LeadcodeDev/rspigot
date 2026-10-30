use crate::{
    Block, BlockId, BlockSettings, FaceSupport, PerState, Property, ShapeId, StateFlags, StateId,
};

pub struct ExposedCutCopperSlab;

impl Block for ExposedCutCopperSlab {
    const NAME: &'static str = "minecraft:exposed_cut_copper_slab";
    const ID: BlockId = BlockId(1159);
    const FIRST_STATE: StateId = StateId(31811);
    const STATE_COUNT: u16 = 6;
    const DEFAULT_STATE: StateId = StateId(31814);
    const PROPERTIES: &'static [Property] = &[
        Property { name: "type", values: &["top", "bottom", "double"] },
        Property { name: "waterlogged", values: &["true", "false"] },
    ];
    const ITEM: Option<u16> = Some(156);
    const WALL_VARIANT: Option<BlockId> = None;
    const SETTINGS: BlockSettings = BlockSettings {
        hardness: 3.0,
        resistance: 6.0,
        friction: 0.6,
        speed_factor: 1.0,
        jump_factor: 1.0,
        requires_correct_tool: true,
        ignited_by_lava: false,
    };
    const LIGHT_EMISSION: PerState<u8> = PerState::Uniform(0);
    const LIGHT_DAMPENING: PerState<u8> = PerState::Varies(&[
        1, 0, 1, 0, 15, 15,
    ]);
    const FLAGS: PerState<StateFlags> = PerState::Varies(&[
        StateFlags(654), StateFlags(558), StateFlags(654), StateFlags(558), StateFlags(1694),
        StateFlags(1566),
    ]);
    const COLLISION: PerState<ShapeId> = PerState::Varies(crate::shapes::COLLISION_WHITE_WOOL_SLAB);
    const FACE_SUPPORT: PerState<FaceSupport> =
        PerState::Varies(crate::shapes::SUPPORT_WHITE_WOOL_SLAB);
    type Behavior = crate::family::Slab;
}
