use crate::{Block, BlockId, BlockSettings, PerState, Property, StateFlags, StateId};

pub struct WaterCauldron;

impl Block for WaterCauldron {
    const NAME: &'static str = "minecraft:water_cauldron";
    const ID: BlockId = BlockId(438);
    const FIRST_STATE: StateId = StateId(11195);
    const STATE_COUNT: u16 = 3;
    const DEFAULT_STATE: StateId = StateId(11195);
    const PROPERTIES: &'static [Property] = &[
        Property { name: "level", values: &["1", "2", "3"] },
    ];
    const ITEM: Option<u16> = Some(1277);
    const WALL_VARIANT: Option<BlockId> = None;
    const SETTINGS: BlockSettings = BlockSettings {
        hardness: 2.0,
        resistance: 2.0,
        friction: 0.6,
        speed_factor: 1.0,
        jump_factor: 1.0,
        requires_correct_tool: true,
        ignited_by_lava: false,
    };
    const LIGHT_EMISSION: PerState<u8> = PerState::Uniform(0);
    const LIGHT_DAMPENING: PerState<u8> = PerState::Uniform(0);
    const FLAGS: PerState<StateFlags> = PerState::Uniform(StateFlags(38));
    type Behavior = crate::family::Plain;
}
