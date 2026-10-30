use crate::{Block, BlockId, BlockSettings, PerState, Property, StateFlags, StateId};

pub struct RedTulip;

impl Block for RedTulip {
    const NAME: &'static str = "minecraft:red_tulip";
    const ID: BlockId = BlockId(207);
    const FIRST_STATE: StateId = StateId(3820);
    const STATE_COUNT: u16 = 1;
    const DEFAULT_STATE: StateId = StateId(3820);
    const PROPERTIES: &'static [Property] = &[];
    const ITEM: Option<u16> = Some(306);
    const WALL_VARIANT: Option<BlockId> = None;
    const SETTINGS: BlockSettings = BlockSettings {
        hardness: 0.0,
        resistance: 0.0,
        friction: 0.6,
        speed_factor: 1.0,
        jump_factor: 1.0,
        requires_correct_tool: false,
        ignited_by_lava: false,
    };
    const LIGHT_EMISSION: PerState<u8> = PerState::Uniform(0);
    const LIGHT_DAMPENING: PerState<u8> = PerState::Uniform(0);
    const FLAGS: PerState<StateFlags> = PerState::Uniform(StateFlags(32));
    type Behavior = crate::family::Plain;
}
