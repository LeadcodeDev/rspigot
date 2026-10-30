use crate::{
    Block, BlockId, BlockSettings, FaceSupport, PerState, Property, ShapeId, SpawnSurface,
    StateFlags, StateId,
};

pub struct MagentaStainedGlass;

impl Block for MagentaStainedGlass {
    const NAME: &'static str = "minecraft:magenta_stained_glass";
    const ID: BlockId = BlockId(351);
    const FIRST_STATE: StateId = StateId(8770);
    const STATE_COUNT: u16 = 1;
    const DEFAULT_STATE: StateId = StateId(8770);
    const PROPERTIES: &'static [Property] = &[];
    const ITEM: Option<u16> = Some(607);
    const WALL_VARIANT: Option<BlockId> = None;
    const SETTINGS: BlockSettings = BlockSettings {
        hardness: 0.3,
        resistance: 0.3,
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
    const FLAGS: PerState<StateFlags> = PerState::Uniform(StateFlags(3110));
    const COLLISION: PerState<ShapeId> = PerState::Uniform(ShapeId(1));
    const FACE_SUPPORT: PerState<FaceSupport> = PerState::Uniform(FaceSupport(63, 63, 63));
    const SPAWN_SURFACE: SpawnSurface = SpawnSurface::Never;
    type Behavior = crate::family::Plain;
}
