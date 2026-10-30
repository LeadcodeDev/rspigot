use crate::{Block, BlockId, BlockSettings, PerState, Property, StateFlags, StateId};

pub struct LightGrayCarpet;

impl Block for LightGrayCarpet {
    const NAME: &'static str = "minecraft:light_gray_carpet";
    const ID: BlockId = BlockId(600);
    const FIRST_STATE: StateId = StateId(14751);
    const STATE_COUNT: u16 = 1;
    const DEFAULT_STATE: StateId = StateId(14751);
    const PROPERTIES: &'static [Property] = &[];
    const ITEM: Option<u16> = Some(588);
    const WALL_VARIANT: Option<BlockId> = None;
    const SETTINGS: BlockSettings = BlockSettings {
        hardness: 0.1,
        resistance: 0.1,
        friction: 0.6,
        speed_factor: 1.0,
        jump_factor: 1.0,
        requires_correct_tool: false,
        ignited_by_lava: true,
    };
    const LIGHT_EMISSION: PerState<u8> = PerState::Uniform(0);
    const LIGHT_DAMPENING: PerState<u8> = PerState::Uniform(0);
    const FLAGS: PerState<StateFlags> = PerState::Uniform(StateFlags(44));
    type Behavior = crate::family::Plain;
}
