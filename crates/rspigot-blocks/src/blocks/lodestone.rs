use crate::{Block, BlockId, BlockSettings, PerState, Property, StateFlags, StateId};

pub struct Lodestone;

impl Block for Lodestone {
    const NAME: &'static str = "minecraft:lodestone";
    const ID: BlockId = BlockId(1013);
    const FIRST_STATE: StateId = StateId(25187);
    const STATE_COUNT: u16 = 1;
    const DEFAULT_STATE: StateId = StateId(25187);
    const PROPERTIES: &'static [Property] = &[];
    const ITEM: Option<u16> = Some(1535);
    const WALL_VARIANT: Option<BlockId> = None;
    const SETTINGS: BlockSettings = BlockSettings {
        hardness: 3.5,
        resistance: 3.5,
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
