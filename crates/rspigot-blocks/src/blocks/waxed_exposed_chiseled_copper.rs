use crate::{Block, BlockId, BlockSettings, PerState, Property, StateFlags, StateId};

pub struct WaxedExposedChiseledCopper;

impl Block for WaxedExposedChiseledCopper {
    const NAME: &'static str = "minecraft:waxed_exposed_chiseled_copper";
    const ID: BlockId = BlockId(1147);
    const FIRST_STATE: StateId = StateId(31162);
    const STATE_COUNT: u16 = 1;
    const DEFAULT_STATE: StateId = StateId(31162);
    const PROPERTIES: &'static [Property] = &[];
    const ITEM: Option<u16> = Some(136);
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
    const FLAGS: PerState<StateFlags> = PerState::Uniform(StateFlags(1054));
    type Behavior = crate::family::Plain;
}
