use crate::{
    Block, BlockId, BlockSettings, FaceSupport, PerState, Property, ShapeId, StateFlags, StateId,
};

pub struct Farmland;

impl Block for Farmland {
    const NAME: &'static str = "minecraft:farmland";
    const ID: BlockId = BlockId(252);
    const FIRST_STATE: StateId = StateId(6875);
    const STATE_COUNT: u16 = 8;
    const DEFAULT_STATE: StateId = StateId(6875);
    const PROPERTIES: &'static [Property] = &[
        Property { name: "moisture", values: &["0", "1", "2", "3", "4", "5", "6", "7"] },
    ];
    const ITEM: Option<u16> = Some(406);
    const WALL_VARIANT: Option<BlockId> = None;
    const SETTINGS: BlockSettings = BlockSettings {
        hardness: 0.6,
        resistance: 0.6,
        friction: 0.6,
        speed_factor: 1.0,
        jump_factor: 1.0,
        requires_correct_tool: false,
        ignited_by_lava: false,
    };
    const LIGHT_EMISSION: PerState<u8> = PerState::Uniform(0);
    const LIGHT_DAMPENING: PerState<u8> = PerState::Uniform(0);
    const FLAGS: PerState<StateFlags> = PerState::Uniform(StateFlags(558));
    const COLLISION: PerState<ShapeId> = PerState::Uniform(ShapeId(64));
    const FACE_SUPPORT: PerState<FaceSupport> = PerState::Uniform(FaceSupport(1, 61, 1));
    type Behavior = crate::family::Plain;
}
