use crate::{Block, BlockId, BlockSettings, PerState, Property, StateFlags, StateId};

pub struct CryingObsidian;

impl Block for CryingObsidian {
    const NAME: &'static str = "minecraft:crying_obsidian";
    const ID: BlockId = BlockId(1007);
    const FIRST_STATE: StateId = StateId(25177);
    const STATE_COUNT: u16 = 1;
    const DEFAULT_STATE: StateId = StateId(25177);
    const PROPERTIES: &'static [Property] = &[];
    const ITEM: Option<u16> = Some(1536);
    const WALL_VARIANT: Option<BlockId> = None;
    const SETTINGS: BlockSettings = BlockSettings {
        hardness: 50.0,
        resistance: 1200.0,
        friction: 0.6,
        speed_factor: 1.0,
        jump_factor: 1.0,
        requires_correct_tool: true,
        ignited_by_lava: false,
    };
    const LIGHT_EMISSION: PerState<u8> = PerState::Uniform(10);
    const LIGHT_DAMPENING: PerState<u8> = PerState::Uniform(15);
    const FLAGS: PerState<StateFlags> = PerState::Uniform(StateFlags(1054));
    type Behavior = crate::family::Plain;
}
