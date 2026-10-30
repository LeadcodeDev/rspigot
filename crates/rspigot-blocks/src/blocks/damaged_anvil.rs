use crate::{
    Block, BlockId, BlockSettings, FaceSupport, PerState, Property, ShapeId, StateFlags, StateId,
};

pub struct DamagedAnvil;

impl Block for DamagedAnvil {
    const NAME: &'static str = "minecraft:damaged_anvil";
    const ID: BlockId = BlockId(522);
    const FIRST_STATE: StateId = StateId(12970);
    const STATE_COUNT: u16 = 4;
    const DEFAULT_STATE: StateId = StateId(12970);
    const PROPERTIES: &'static [Property] = &[
        Property { name: "facing", values: &["north", "south", "west", "east"] },
    ];
    const ITEM: Option<u16> = Some(555);
    const WALL_VARIANT: Option<BlockId> = None;
    const SETTINGS: BlockSettings = BlockSettings {
        hardness: 5.0,
        resistance: 1200.0,
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
    const FLAGS: PerState<StateFlags> = PerState::Uniform(StateFlags(2094));
    const COLLISION: PerState<ShapeId> = PerState::Varies(crate::shapes::COLLISION_ANVIL);
    const FACE_SUPPORT: PerState<FaceSupport> = PerState::Uniform(FaceSupport(0, 3, 0));
    type Behavior = crate::family::Anvil;
}
