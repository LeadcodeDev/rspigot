use crate::{
    Block, BlockId, BlockSettings, FaceSupport, PerState, Property, ShapeId, StateFlags, StateId,
};

pub struct BirchPressurePlate;

impl Block for BirchPressurePlate {
    const NAME: &'static str = "minecraft:birch_pressure_plate";
    const ID: BlockId = BlockId(311);
    const FIRST_STATE: StateId = StateId(8533);
    const STATE_COUNT: u16 = 2;
    const DEFAULT_STATE: StateId = StateId(8534);
    const PROPERTIES: &'static [Property] = &[
        Property { name: "powered", values: &["true", "false"] },
    ];
    const ITEM: Option<u16> = Some(877);
    const WALL_VARIANT: Option<BlockId> = None;
    const SETTINGS: BlockSettings = BlockSettings {
        hardness: 0.5,
        resistance: 0.5,
        friction: 0.6,
        speed_factor: 1.0,
        jump_factor: 1.0,
        bounce_restitution: 0.0,
        fall_distance_reduction: 0.0,
        requires_correct_tool: false,
        ignited_by_lava: true,
    };
    const LIGHT_EMISSION: PerState<u8> = PerState::Uniform(0);
    const LIGHT_DAMPENING: PerState<u8> = PerState::Uniform(0);
    const FLAGS: PerState<StateFlags> = PerState::Uniform(StateFlags(34));
    const COLLISION: PerState<ShapeId> = PerState::Uniform(ShapeId(0));
    const FACE_SUPPORT: PerState<FaceSupport> = PerState::Uniform(FaceSupport(0, 0, 0));
    type Behavior = crate::family::Plain;
}
