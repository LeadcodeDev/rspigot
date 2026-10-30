use crate::{
    Block, BlockId, BlockSettings, FaceSupport, PerState, Property, ShapeId, StateFlags, StateId,
};

pub struct TurtleEgg;

impl Block for TurtleEgg {
    const NAME: &'static str = "minecraft:turtle_egg";
    const ID: BlockId = BlockId(835);
    const FIRST_STATE: StateId = StateId(18447);
    const STATE_COUNT: u16 = 12;
    const DEFAULT_STATE: StateId = StateId(18447);
    const PROPERTIES: &'static [Property] = &[
        Property { name: "eggs", values: &["1", "2", "3", "4"] },
        Property { name: "hatch", values: &["0", "1", "2"] },
    ];
    const ITEM: Option<u16> = Some(753);
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
        ignited_by_lava: false,
    };
    const LIGHT_EMISSION: PerState<u8> = PerState::Uniform(0);
    const LIGHT_DAMPENING: PerState<u8> = PerState::Uniform(0);
    const FLAGS: PerState<StateFlags> = PerState::Uniform(StateFlags(2598));
    const COLLISION: PerState<ShapeId> = PerState::Varies(crate::shapes::COLLISION_TURTLE_EGG);
    const FACE_SUPPORT: PerState<FaceSupport> = PerState::Uniform(FaceSupport(0, 1, 0));
    type Behavior = crate::family::Plain;
}
