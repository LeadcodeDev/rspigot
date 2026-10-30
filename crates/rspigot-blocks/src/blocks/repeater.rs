use crate::{Block, BlockId, BlockSettings, PerState, Property, StateFlags, StateId};

pub struct Repeater;

impl Block for Repeater {
    const NAME: &'static str = "minecraft:repeater";
    const ID: BlockId = BlockId(348);
    const FIRST_STATE: StateId = StateId(8704);
    const STATE_COUNT: u16 = 64;
    const DEFAULT_STATE: StateId = StateId(8707);
    const PROPERTIES: &'static [Property] = &[
        Property { name: "delay", values: &["1", "2", "3", "4"] },
        Property { name: "facing", values: &["north", "south", "west", "east"] },
        Property { name: "locked", values: &["true", "false"] },
        Property { name: "powered", values: &["true", "false"] },
    ];
    const ITEM: Option<u16> = Some(827);
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
    const FLAGS: PerState<StateFlags> = PerState::Uniform(StateFlags(44));
    type Behavior = crate::family::HorizontalAway;
}
