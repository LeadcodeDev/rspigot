use crate::{Block, BlockId, BlockSettings, PerState, Property, StateFlags, StateId};

pub struct PowderSnow;

impl Block for PowderSnow {
    const NAME: &'static str = "minecraft:powder_snow";
    const ID: BlockId = BlockId(1117);
    const FIRST_STATE: StateId = StateId(30519);
    const STATE_COUNT: u16 = 1;
    const DEFAULT_STATE: StateId = StateId(30519);
    const PROPERTIES: &'static [Property] = &[];
    const ITEM: Option<u16> = Some(1131);
    const WALL_VARIANT: Option<BlockId> = None;
    const SETTINGS: BlockSettings = BlockSettings {
        hardness: 0.25,
        resistance: 0.25,
        friction: 0.6,
        speed_factor: 1.0,
        jump_factor: 1.0,
        requires_correct_tool: false,
        ignited_by_lava: false,
    };
    const LIGHT_EMISSION: PerState<u8> = PerState::Uniform(0);
    const LIGHT_DAMPENING: PerState<u8> = PerState::Uniform(1);
    const FLAGS: PerState<StateFlags> = PerState::Uniform(StateFlags(0));
    type Behavior = crate::family::Plain;
}
