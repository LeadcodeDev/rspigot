use crate::{Block, BlockId, BlockSettings, PerState, Property, StateFlags, StateId};

pub struct Beetroots;

impl Block for Beetroots {
    const NAME: &'static str = "minecraft:beetroots";
    const ID: BlockId = BlockId(723);
    const FIRST_STATE: StateId = StateId(16792);
    const STATE_COUNT: u16 = 4;
    const DEFAULT_STATE: StateId = StateId(16792);
    const PROPERTIES: &'static [Property] = &[
        Property { name: "age", values: &["0", "1", "2", "3"] },
    ];
    const ITEM: Option<u16> = Some(1439);
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
    const FLAGS: PerState<StateFlags> = PerState::Varies(&[
        StateFlags(544), StateFlags(544), StateFlags(544), StateFlags(32),
    ]);
    type Behavior = crate::family::Plain;
}
