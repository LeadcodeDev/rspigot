use crate::{Block, BlockId, BlockSettings, PerState, Property, StateFlags, StateId};

pub struct Stonecutter;

impl Block for Stonecutter {
    const NAME: &'static str = "minecraft:stonecutter";
    const ID: BlockId = BlockId(937);
    const FIRST_STATE: StateId = StateId(24158);
    const STATE_COUNT: u16 = 4;
    const DEFAULT_STATE: StateId = StateId(24158);
    const PROPERTIES: &'static [Property] = &[
        Property { name: "facing", values: &["north", "south", "west", "east"] },
    ];
    const ITEM: Option<u16> = Some(1513);
    const WALL_VARIANT: Option<BlockId> = None;
    const SETTINGS: BlockSettings = BlockSettings {
        hardness: 3.5,
        resistance: 3.5,
        friction: 0.6,
        speed_factor: 1.0,
        jump_factor: 1.0,
        requires_correct_tool: true,
        ignited_by_lava: false,
    };
    const LIGHT_EMISSION: PerState<u8> = PerState::Uniform(0);
    const LIGHT_DAMPENING: PerState<u8> = PerState::Uniform(0);
    const FLAGS: PerState<StateFlags> = PerState::Uniform(StateFlags(46));
    type Behavior = crate::family::HorizontalAway;
}
