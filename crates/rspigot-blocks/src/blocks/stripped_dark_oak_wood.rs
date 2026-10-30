use crate::{
    Block, BlockId, BlockSettings, FaceSupport, PerState, Property, ShapeId, StateFlags, StateId,
};

pub struct StrippedDarkOakWood;

impl Block for StrippedDarkOakWood {
    const NAME: &'static str = "minecraft:stripped_dark_oak_wood";
    const ID: BlockId = BlockId(90);
    const FIRST_STATE: StateId = StateId(255);
    const STATE_COUNT: u16 = 3;
    const DEFAULT_STATE: StateId = StateId(256);
    const PROPERTIES: &'static [Property] = &[
        Property { name: "axis", values: &["x", "y", "z"] },
    ];
    const ITEM: Option<u16> = Some(196);
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
    const LIGHT_EMISSION: PerState<u8> = PerState::Uniform(0);
    const LIGHT_DAMPENING: PerState<u8> = PerState::Uniform(15);
    const FLAGS: PerState<StateFlags> = PerState::Uniform(StateFlags(7198));
    const COLLISION: PerState<ShapeId> = PerState::Uniform(ShapeId(1));
    const FACE_SUPPORT: PerState<FaceSupport> = PerState::Uniform(FaceSupport(63, 63, 63));
    type Behavior = crate::family::Pillar;
}
