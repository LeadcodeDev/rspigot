use crate::{
    Block, BlockId, BlockSettings, FaceSupport, PerState, Property, ShapeId, StateFlags, StateId,
};

pub struct Wheat;

impl Block for Wheat {
    const NAME: &'static str = "minecraft:wheat";
    const ID: BlockId = BlockId(251);
    const FIRST_STATE: StateId = StateId(6867);
    const STATE_COUNT: u16 = 8;
    const DEFAULT_STATE: StateId = StateId(6867);
    const PROPERTIES: &'static [Property] = &[
        Property { name: "age", values: &["0", "1", "2", "3", "4", "5", "6", "7"] },
    ];
    const ITEM: Option<u16> = Some(1065);
    const WALL_VARIANT: Option<BlockId> = None;
    const SETTINGS: BlockSettings = BlockSettings {
        hardness: 0.0,
        resistance: 0.0,
        friction: 0.6,
        speed_factor: 1.0,
        jump_factor: 1.0,
        bounce_restitution: 0.0,
        fall_distance_reduction: 0.0,
        requires_correct_tool: false,
        ignited_by_lava: false,
    };
    const LIGHT_EMISSION: PerState<u8> = PerState::Uniform(0);
    const LIGHT_DAMPENING: PerState<u8> = PerState::Uniform(0);
    const FLAGS: PerState<StateFlags> = PerState::Varies(&[
        StateFlags(544), StateFlags(544), StateFlags(544), StateFlags(544), StateFlags(544),
        StateFlags(544), StateFlags(544), StateFlags(32),
    ]);
    const COLLISION: PerState<ShapeId> = PerState::Uniform(ShapeId(0));
    const FACE_SUPPORT: PerState<FaceSupport> = PerState::Uniform(FaceSupport(0, 0, 0));
    type Behavior = crate::family::Plain;
}
