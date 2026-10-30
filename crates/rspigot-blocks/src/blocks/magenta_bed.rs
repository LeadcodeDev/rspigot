use crate::{Block, BlockId, BlockSettings, PerState, Property, StateFlags, StateId};

pub struct MagentaBed;

impl Block for MagentaBed {
    const NAME: &'static str = "minecraft:magenta_bed";
    const ID: BlockId = BlockId(121);
    const FIRST_STATE: StateId = StateId(2062);
    const STATE_COUNT: u16 = 16;
    const DEFAULT_STATE: StateId = StateId(2065);
    const PROPERTIES: &'static [Property] = &[
        Property { name: "facing", values: &["north", "south", "west", "east"] },
        Property { name: "occupied", values: &["true", "false"] },
        Property { name: "part", values: &["head", "foot"] },
    ];
    const ITEM: Option<u16> = Some(1221);
    const WALL_VARIANT: Option<BlockId> = None;
    const SETTINGS: BlockSettings = BlockSettings {
        hardness: 0.2,
        resistance: 0.2,
        friction: 0.6,
        speed_factor: 1.0,
        jump_factor: 1.0,
        requires_correct_tool: false,
        ignited_by_lava: true,
    };
    const LIGHT_EMISSION: PerState<u8> = PerState::Uniform(0);
    const LIGHT_DAMPENING: PerState<u8> = PerState::Uniform(0);
    const FLAGS: PerState<StateFlags> = PerState::Uniform(StateFlags(38));
    type Behavior = crate::family::HorizontalToward;
}
