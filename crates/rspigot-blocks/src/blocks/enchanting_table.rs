use crate::{
    Block, BlockId, BlockSettings, FaceSupport, PerState, Property, ShapeId, StateFlags, StateId,
};

pub struct EnchantingTable;

impl Block for EnchantingTable {
    const NAME: &'static str = "minecraft:enchanting_table";
    const ID: BlockId = BlockId(435);
    const FIRST_STATE: StateId = StateId(11185);
    const STATE_COUNT: u16 = 1;
    const DEFAULT_STATE: StateId = StateId(11185);
    const PROPERTIES: &'static [Property] = &[];
    const ITEM: Option<u16> = Some(507);
    const WALL_VARIANT: Option<BlockId> = None;
    const SETTINGS: BlockSettings = BlockSettings {
        hardness: 5.0,
        resistance: 1200.0,
        friction: 0.6,
        speed_factor: 1.0,
        jump_factor: 1.0,
        requires_correct_tool: true,
        ignited_by_lava: false,
    };
    const LIGHT_EMISSION: PerState<u8> = PerState::Uniform(7);
    const LIGHT_DAMPENING: PerState<u8> = PerState::Uniform(0);
    const FLAGS: PerState<StateFlags> = PerState::Uniform(StateFlags(46));
    const COLLISION: PerState<ShapeId> = PerState::Uniform(ShapeId(15));
    const FACE_SUPPORT: PerState<FaceSupport> = PerState::Uniform(FaceSupport(1, 61, 1));
    type Behavior = crate::family::Plain;
}
