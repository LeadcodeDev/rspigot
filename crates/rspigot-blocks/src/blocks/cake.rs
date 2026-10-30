use crate::{
    Block, BlockId, BlockSettings, FaceSupport, PerState, Property, ShapeId, StateFlags, StateId,
};

pub struct Cake;

impl Block for Cake {
    const NAME: &'static str = "minecraft:cake";
    const ID: BlockId = BlockId(347);
    const FIRST_STATE: StateId = StateId(8697);
    const STATE_COUNT: u16 = 7;
    const DEFAULT_STATE: StateId = StateId(8697);
    const PROPERTIES: &'static [Property] = &[
        Property { name: "bites", values: &["0", "1", "2", "3", "4", "5", "6"] },
    ];
    const ITEM: Option<u16> = Some(1218);
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
    const FLAGS: PerState<StateFlags> = PerState::Uniform(StateFlags(46));
    const COLLISION: PerState<ShapeId> = PerState::Varies(crate::shapes::COLLISION_CAKE);
    const FACE_SUPPORT: PerState<FaceSupport> = PerState::Varies(crate::shapes::SUPPORT_CAKE);
    type Behavior = crate::family::Plain;
}
