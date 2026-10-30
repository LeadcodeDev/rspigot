use crate::{Block, BlockId, BlockSettings, PerState, Property, StateFlags, StateId};

pub struct IronBlock;

impl Block for IronBlock {
    const NAME: &'static str = "minecraft:iron_block";
    const ID: BlockId = BlockId(218);
    const FIRST_STATE: StateId = StateId(3831);
    const STATE_COUNT: u16 = 1;
    const DEFAULT_STATE: StateId = StateId(3831);
    const PROPERTIES: &'static [Property] = &[];
    const ITEM: Option<u16> = Some(119);
    const WALL_VARIANT: Option<BlockId> = None;
    const SETTINGS: BlockSettings = BlockSettings {
        hardness: 5.0,
        resistance: 6.0,
        friction: 0.6,
        speed_factor: 1.0,
        jump_factor: 1.0,
        requires_correct_tool: true,
        ignited_by_lava: false,
    };
    const LIGHT_EMISSION: PerState<u8> = PerState::Uniform(0);
    const LIGHT_DAMPENING: PerState<u8> = PerState::Uniform(15);
    const FLAGS: PerState<StateFlags> = PerState::Uniform(StateFlags(1054));
    type Behavior = crate::family::Plain;
}
