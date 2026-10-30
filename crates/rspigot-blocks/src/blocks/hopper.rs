use crate::{
    Block, BlockId, BlockSettings, FaceSupport, PerState, Property, ShapeId, StateFlags, StateId,
};

pub struct Hopper;

impl Block for Hopper {
    const NAME: &'static str = "minecraft:hopper";
    const ID: BlockId = BlockId(530);
    const FIRST_STATE: StateId = StateId(13080);
    const STATE_COUNT: u16 = 10;
    const DEFAULT_STATE: StateId = StateId(13080);
    const PROPERTIES: &'static [Property] = &[
        Property { name: "enabled", values: &["true", "false"] },
        Property { name: "facing", values: &["down", "north", "south", "west", "east"] },
    ];
    const ITEM: Option<u16> = Some(834);
    const WALL_VARIANT: Option<BlockId> = None;
    const SETTINGS: BlockSettings = BlockSettings {
        hardness: 3.0,
        resistance: 4.8,
        friction: 0.6,
        speed_factor: 1.0,
        jump_factor: 1.0,
        bounce_restitution: 0.0,
        fall_distance_reduction: 0.0,
        requires_correct_tool: true,
        ignited_by_lava: false,
    };
    const LIGHT_EMISSION: PerState<u8> = PerState::Uniform(0);
    const LIGHT_DAMPENING: PerState<u8> = PerState::Uniform(0);
    const FLAGS: PerState<StateFlags> = PerState::Uniform(StateFlags(2086));
    const COLLISION: PerState<ShapeId> = PerState::Varies(crate::shapes::COLLISION_HOPPER);
    const FACE_SUPPORT: PerState<FaceSupport> = PerState::Varies(crate::shapes::SUPPORT_HOPPER);
    type Behavior = crate::family::Plain;
}
