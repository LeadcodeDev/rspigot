use crate::{Block, BlockId, BlockSettings, PerState, Property, StateFlags, StateId};

pub struct Beacon;

impl Block for Beacon {
    const NAME: &'static str = "minecraft:beacon";
    const ID: BlockId = BlockId(459);
    const FIRST_STATE: StateId = StateId(11722);
    const STATE_COUNT: u16 = 1;
    const DEFAULT_STATE: StateId = StateId(11722);
    const PROPERTIES: &'static [Property] = &[];
    const ITEM: Option<u16> = Some(530);
    const WALL_VARIANT: Option<BlockId> = None;
    const SETTINGS: BlockSettings = BlockSettings {
        hardness: 3.0,
        resistance: 3.0,
        friction: 0.6,
        speed_factor: 1.0,
        jump_factor: 1.0,
        requires_correct_tool: false,
        ignited_by_lava: false,
    };
    const LIGHT_EMISSION: PerState<u8> = PerState::Uniform(15);
    const LIGHT_DAMPENING: PerState<u8> = PerState::Uniform(1);
    const FLAGS: PerState<StateFlags> = PerState::Uniform(StateFlags(1030));
    type Behavior = crate::family::Plain;
}
