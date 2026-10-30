use crate::{Block, BlockId, BlockSettings, PerState, Property, StateFlags, StateId};

pub struct CherryButton;

impl Block for CherryButton {
    const NAME: &'static str = "minecraft:cherry_button";
    const ID: BlockId = BlockId(500);
    const FIRST_STATE: StateId = StateId(12538);
    const STATE_COUNT: u16 = 24;
    const DEFAULT_STATE: StateId = StateId(12547);
    const PROPERTIES: &'static [Property] = &[
        Property { name: "face", values: &["floor", "wall", "ceiling"] },
        Property { name: "facing", values: &["north", "south", "west", "east"] },
        Property { name: "powered", values: &["true", "false"] },
    ];
    const ITEM: Option<u16> = Some(863);
    const WALL_VARIANT: Option<BlockId> = None;
    const SETTINGS: BlockSettings = BlockSettings {
        hardness: 0.5,
        resistance: 0.5,
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
