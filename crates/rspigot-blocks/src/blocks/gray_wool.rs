use crate::{Block, BlockId, BlockSettings, PerState, Property, StateFlags, StateId};

pub struct GrayWool;

impl Block for GrayWool {
    const NAME: &'static str = "minecraft:gray_wool";
    const ID: BlockId = BlockId(158);
    const FIRST_STATE: StateId = StateId(2416);
    const STATE_COUNT: u16 = 1;
    const DEFAULT_STATE: StateId = StateId(2416);
    const PROPERTIES: &'static [Property] = &[];
    const ITEM: Option<u16> = Some(257);
    const WALL_VARIANT: Option<BlockId> = None;
    const SETTINGS: BlockSettings = BlockSettings {
        hardness: 0.8,
        resistance: 0.8,
        friction: 0.6,
        speed_factor: 1.0,
        jump_factor: 1.0,
        requires_correct_tool: false,
        ignited_by_lava: true,
    };
    const LIGHT_EMISSION: PerState<u8> = PerState::Uniform(0);
    const LIGHT_DAMPENING: PerState<u8> = PerState::Uniform(15);
    const FLAGS: PerState<StateFlags> = PerState::Uniform(StateFlags(1054));
    type Behavior = crate::family::Plain;
}
