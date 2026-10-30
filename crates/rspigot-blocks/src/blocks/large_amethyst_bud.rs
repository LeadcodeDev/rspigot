use crate::{
    Block, BlockId, BlockSettings, FaceSupport, PerState, Property, ShapeId, StateFlags, StateId,
};

pub struct LargeAmethystBud;

impl Block for LargeAmethystBud {
    const NAME: &'static str = "minecraft:large_amethyst_bud";
    const ID: BlockId = BlockId(1071);
    const FIRST_STATE: StateId = StateId(26773);
    const STATE_COUNT: u16 = 12;
    const DEFAULT_STATE: StateId = StateId(26782);
    const PROPERTIES: &'static [Property] = &[
        Property { name: "facing", values: &["north", "east", "south", "west", "up", "down"] },
        Property { name: "waterlogged", values: &["true", "false"] },
    ];
    const ITEM: Option<u16> = Some(1569);
    const WALL_VARIANT: Option<BlockId> = None;
    const SETTINGS: BlockSettings = BlockSettings {
        hardness: 1.5,
        resistance: 1.5,
        friction: 0.6,
        speed_factor: 1.0,
        jump_factor: 1.0,
        bounce_restitution: 0.0,
        fall_distance_reduction: 0.0,
        requires_correct_tool: false,
        ignited_by_lava: false,
    };
    const LIGHT_EMISSION: PerState<u8> = PerState::Uniform(4);
    const LIGHT_DAMPENING: PerState<u8> = PerState::Varies(&[
        1, 0, 1, 0, 1, 0, 1, 0, 1, 0, 1, 0,
    ]);
    const FLAGS: PerState<StateFlags> = PerState::Varies(&[
        StateFlags(18566), StateFlags(18470), StateFlags(18566), StateFlags(18470),
        StateFlags(18566), StateFlags(18470), StateFlags(18566), StateFlags(18470),
        StateFlags(18566), StateFlags(18470), StateFlags(18566), StateFlags(18470),
    ]);
    const COLLISION: PerState<ShapeId> =
        PerState::Varies(crate::shapes::COLLISION_LARGE_AMETHYST_BUD);
    const FACE_SUPPORT: PerState<FaceSupport> =
        PerState::Varies(crate::shapes::SUPPORT_AMETHYST_CLUSTER);
    type Behavior = crate::family::FaceAttached;
}
