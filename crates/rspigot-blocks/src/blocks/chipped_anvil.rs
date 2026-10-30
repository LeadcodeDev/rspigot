use crate::{Block, BlockId, BlockSettings, PerState, Property, StateFlags, StateId};

pub struct ChippedAnvil;

impl Block for ChippedAnvil {
    const NAME: &'static str = "minecraft:chipped_anvil";
    const ID: BlockId = BlockId(521);
    const FIRST_STATE: StateId = StateId(12966);
    const STATE_COUNT: u16 = 4;
    const DEFAULT_STATE: StateId = StateId(12966);
    const PROPERTIES: &'static [Property] = &[
        Property { name: "facing", values: &["north", "south", "west", "east"] },
    ];
    const ITEM: Option<u16> = Some(554);
    const WALL_VARIANT: Option<BlockId> = None;
    const SETTINGS: BlockSettings = BlockSettings {
        hardness: 5.0,
        resistance: 1200.0,
        friction: 0.6,
        speed_factor: 1.0,
        jump_factor: 1.0,
        requires_correct_tool: true,
        ignited_by_lava: false,
    };
    const LIGHT_EMISSION: PerState<u8> = PerState::Uniform(0);
    const LIGHT_DAMPENING: PerState<u8> = PerState::Uniform(0);
    const FLAGS: PerState<StateFlags> = PerState::Uniform(StateFlags(46));
    type Behavior = crate::family::Plain;
}
