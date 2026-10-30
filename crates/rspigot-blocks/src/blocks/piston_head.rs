use crate::{
    Block, BlockId, BlockSettings, FaceSupport, PerState, Property, ShapeId, StateFlags, StateId,
};

pub struct PistonHead;

impl Block for PistonHead {
    const NAME: &'static str = "minecraft:piston_head";
    const ID: BlockId = BlockId(150);
    const FIRST_STATE: StateId = StateId(2385);
    const STATE_COUNT: u16 = 24;
    const DEFAULT_STATE: StateId = StateId(2387);
    const PROPERTIES: &'static [Property] = &[
        Property { name: "facing", values: &["north", "east", "south", "west", "up", "down"] },
        Property { name: "short", values: &["true", "false"] },
        Property { name: "type", values: &["normal", "sticky"] },
    ];
    const ITEM: Option<u16> = None;
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
    const LIGHT_EMISSION: PerState<u8> = PerState::Uniform(0);
    const LIGHT_DAMPENING: PerState<u8> = PerState::Uniform(0);
    const FLAGS: PerState<StateFlags> = PerState::Uniform(StateFlags(46));
    const COLLISION: PerState<ShapeId> = PerState::Varies(crate::shapes::COLLISION_PISTON_HEAD);
    const FACE_SUPPORT: PerState<FaceSupport> =
        PerState::Varies(crate::shapes::SUPPORT_PISTON_HEAD);
    type Behavior = crate::family::Plain;
}
