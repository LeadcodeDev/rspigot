use crate::{Block, BlockId, BlockSettings, PerState, Property, StateFlags, StateId};

pub struct DirtPath;

impl Block for DirtPath {
    const NAME: &'static str = "minecraft:dirt_path";
    const ID: BlockId = BlockId(724);
    const FIRST_STATE: StateId = StateId(16796);
    const STATE_COUNT: u16 = 1;
    const DEFAULT_STATE: StateId = StateId(16796);
    const PROPERTIES: &'static [Property] = &[];
    const ITEM: Option<u16> = Some(598);
    const WALL_VARIANT: Option<BlockId> = None;
    const SETTINGS: BlockSettings = BlockSettings {
        hardness: 0.65,
        resistance: 0.65,
        friction: 0.6,
        speed_factor: 1.0,
        jump_factor: 1.0,
        requires_correct_tool: false,
        ignited_by_lava: false,
    };
    const LIGHT_EMISSION: PerState<u8> = PerState::Uniform(0);
    const LIGHT_DAMPENING: PerState<u8> = PerState::Uniform(0);
    const FLAGS: PerState<StateFlags> = PerState::Uniform(StateFlags(46));
    type Behavior = crate::family::Plain;
}
