use crate::{Block, BlockId, BlockSettings, PerState, Property, StateFlags, StateId};

pub struct RedGlazedTerracotta;

impl Block for RedGlazedTerracotta {
    const NAME: &'static str = "minecraft:red_glazed_terracotta";
    const ID: BlockId = BlockId(766);
    const FIRST_STATE: StateId = StateId(17003);
    const STATE_COUNT: u16 = 4;
    const DEFAULT_STATE: StateId = StateId(17003);
    const PROPERTIES: &'static [Property] = &[
        Property { name: "facing", values: &["north", "south", "west", "east"] },
    ];
    const ITEM: Option<u16> = Some(687);
    const WALL_VARIANT: Option<BlockId> = None;
    const SETTINGS: BlockSettings = BlockSettings {
        hardness: 1.4,
        resistance: 1.4,
        friction: 0.6,
        speed_factor: 1.0,
        jump_factor: 1.0,
        requires_correct_tool: true,
        ignited_by_lava: false,
    };
    const LIGHT_EMISSION: PerState<u8> = PerState::Uniform(0);
    const LIGHT_DAMPENING: PerState<u8> = PerState::Uniform(15);
    const FLAGS: PerState<StateFlags> = PerState::Uniform(StateFlags(1054));
    type Behavior = crate::family::HorizontalAway;
}
