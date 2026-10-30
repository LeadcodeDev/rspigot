use crate::{
    Block, BlockId, BlockSettings, FaceSupport, PerState, Property, ShapeId, StateFlags, StateId,
};

pub struct CrimsonPlanks;

impl Block for CrimsonPlanks {
    const NAME: &'static str = "minecraft:crimson_planks";
    const ID: BlockId = BlockId(973);
    const FIRST_STATE: StateId = StateId(24389);
    const STATE_COUNT: u16 = 1;
    const DEFAULT_STATE: StateId = StateId(24389);
    const PROPERTIES: &'static [Property] = &[];
    const ITEM: Option<u16> = Some(74);
    const WALL_VARIANT: Option<BlockId> = None;
    const SETTINGS: BlockSettings = BlockSettings {
        hardness: 2.0,
        resistance: 3.0,
        friction: 0.6,
        speed_factor: 1.0,
        jump_factor: 1.0,
        requires_correct_tool: false,
        ignited_by_lava: false,
    };
    const LIGHT_EMISSION: PerState<u8> = PerState::Uniform(0);
    const LIGHT_DAMPENING: PerState<u8> = PerState::Uniform(15);
    const FLAGS: PerState<StateFlags> = PerState::Uniform(StateFlags(1054));
    const COLLISION: PerState<ShapeId> = PerState::Uniform(ShapeId(1));
    const FACE_SUPPORT: PerState<FaceSupport> = PerState::Uniform(FaceSupport(63, 63, 63));
    type Behavior = crate::family::Plain;
}
