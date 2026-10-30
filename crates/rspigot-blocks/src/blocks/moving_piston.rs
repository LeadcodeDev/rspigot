use crate::{
    Block, BlockId, BlockSettings, FaceSupport, PerState, Property, ShapeId, StateFlags, StateId,
};

pub struct MovingPiston;

impl Block for MovingPiston {
    const NAME: &'static str = "minecraft:moving_piston";
    const ID: BlockId = BlockId(199);
    const FIRST_STATE: StateId = StateId(3801);
    const STATE_COUNT: u16 = 12;
    const DEFAULT_STATE: StateId = StateId(3801);
    const PROPERTIES: &'static [Property] = &[
        Property { name: "facing", values: &["north", "east", "south", "west", "up", "down"] },
        Property { name: "type", values: &["normal", "sticky"] },
    ];
    const ITEM: Option<u16> = None;
    const WALL_VARIANT: Option<BlockId> = None;
    const SETTINGS: BlockSettings = BlockSettings {
        hardness: -1.0,
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
    const FLAGS: PerState<StateFlags> = PerState::Uniform(StateFlags(34));
    const COLLISION: PerState<ShapeId> = PerState::Uniform(ShapeId(0));
    const FACE_SUPPORT: PerState<FaceSupport> = PerState::Uniform(FaceSupport(0, 0, 0));
    type Behavior = crate::family::Plain;
}
