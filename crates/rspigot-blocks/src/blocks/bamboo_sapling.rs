use crate::{Block, BlockId, BlockSettings, PerState, Property, StateFlags, StateId};

pub struct BambooSapling;

impl Block for BambooSapling {
    const NAME: &'static str = "minecraft:bamboo_sapling";
    const ID: BlockId = BlockId(881);
    const FIRST_STATE: StateId = StateId(18635);
    const STATE_COUNT: u16 = 1;
    const DEFAULT_STATE: StateId = StateId(18635);
    const PROPERTIES: &'static [Property] = &[];
    const ITEM: Option<u16> = None;
    const WALL_VARIANT: Option<BlockId> = None;
    const SETTINGS: BlockSettings = BlockSettings {
        hardness: 1.0,
        resistance: 1.0,
        friction: 0.6,
        speed_factor: 1.0,
        jump_factor: 1.0,
        requires_correct_tool: false,
        ignited_by_lava: true,
    };
    const LIGHT_EMISSION: PerState<u8> = PerState::Uniform(0);
    const LIGHT_DAMPENING: PerState<u8> = PerState::Uniform(0);
    const FLAGS: PerState<StateFlags> = PerState::Uniform(StateFlags(544));
    type Behavior = crate::family::Plain;
}
