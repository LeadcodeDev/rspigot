use crate::{
    Block, BlockId, BlockSettings, FaceSupport, PerState, Property, ShapeId, StateFlags, StateId,
};

pub struct EndRod;

impl Block for EndRod {
    const NAME: &'static str = "minecraft:end_rod";
    const ID: BlockId = BlockId(712);
    const FIRST_STATE: StateId = StateId(16611);
    const STATE_COUNT: u16 = 6;
    const DEFAULT_STATE: StateId = StateId(16615);
    const PROPERTIES: &'static [Property] = &[
        Property { name: "facing", values: &["north", "east", "south", "west", "up", "down"] },
    ];
    const ITEM: Option<u16> = Some(396);
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
    const LIGHT_EMISSION: PerState<u8> = PerState::Uniform(14);
    const LIGHT_DAMPENING: PerState<u8> = PerState::Uniform(0);
    const FLAGS: PerState<StateFlags> = PerState::Uniform(StateFlags(36));
    const COLLISION: PerState<ShapeId> = PerState::Varies(crate::shapes::COLLISION_END_ROD);
    const FACE_SUPPORT: PerState<FaceSupport> = PerState::Varies(crate::shapes::SUPPORT_END_ROD);
    type Behavior = crate::family::FaceAttached;
}
