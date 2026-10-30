use crate::{Block, BlockId, BlockSettings, PerState, Property, StateFlags, StateId};

pub struct CyanCarpet;

impl Block for CyanCarpet {
    const NAME: &'static str = "minecraft:cyan_carpet";
    const ID: BlockId = BlockId(601);
    const FIRST_STATE: StateId = StateId(14752);
    const STATE_COUNT: u16 = 1;
    const DEFAULT_STATE: StateId = StateId(14752);
    const PROPERTIES: &'static [Property] = &[];
    const ITEM: Option<u16> = Some(589);
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
