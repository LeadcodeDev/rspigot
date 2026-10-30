use crate::{Block, BlockId, BlockSettings, PerState, Property, StateFlags, StateId};

pub struct StoneButton;

impl Block for StoneButton {
    const NAME: &'static str = "minecraft:stone_button";
    const ID: BlockId = BlockId(324);
    const FIRST_STATE: StateId = StateId(8565);
    const STATE_COUNT: u16 = 24;
    const DEFAULT_STATE: StateId = StateId(8574);
    const PROPERTIES: &'static [Property] = &[
        Property { name: "face", values: &["floor", "wall", "ceiling"] },
        Property { name: "facing", values: &["north", "south", "west", "east"] },
        Property { name: "powered", values: &["true", "false"] },
    ];
    const ITEM: Option<u16> = Some(856);
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
