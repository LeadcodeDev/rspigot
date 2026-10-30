use crate::{Block, BlockId, BlockSettings, PerState, Property, StateFlags, StateId};

pub struct WeatheredCutCopper;

impl Block for WeatheredCutCopper {
    const NAME: &'static str = "minecraft:weathered_cut_copper";
    const ID: BlockId = BlockId(1136);
    const FIRST_STATE: StateId = StateId(31151);
    const STATE_COUNT: u16 = 1;
    const DEFAULT_STATE: StateId = StateId(31151);
    const PROPERTIES: &'static [Property] = &[];
    const ITEM: Option<u16> = Some(141);
    const WALL_VARIANT: Option<BlockId> = None;
    const SETTINGS: BlockSettings = BlockSettings {
        hardness: 3.0,
        resistance: 6.0,
        friction: 0.6,
        speed_factor: 1.0,
        jump_factor: 1.0,
        requires_correct_tool: true,
        ignited_by_lava: false,
    };
    const LIGHT_EMISSION: PerState<u8> = PerState::Uniform(0);
    const LIGHT_DAMPENING: PerState<u8> = PerState::Uniform(15);
    const FLAGS: PerState<StateFlags> = PerState::Uniform(StateFlags(1566));
    type Behavior = crate::family::Plain;
}
