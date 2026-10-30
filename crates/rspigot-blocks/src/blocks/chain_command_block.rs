use crate::{Block, BlockId, BlockSettings, PerState, Property, StateFlags, StateId};

pub struct ChainCommandBlock;

impl Block for ChainCommandBlock {
    const NAME: &'static str = "minecraft:chain_command_block";
    const ID: BlockId = BlockId(727);
    const FIRST_STATE: StateId = StateId(16810);
    const STATE_COUNT: u16 = 12;
    const DEFAULT_STATE: StateId = StateId(16816);
    const PROPERTIES: &'static [Property] = &[
        Property { name: "conditional", values: &["true", "false"] },
        Property { name: "facing", values: &["north", "east", "south", "west", "up", "down"] },
    ];
    const ITEM: Option<u16> = Some(649);
    const WALL_VARIANT: Option<BlockId> = None;
    const SETTINGS: BlockSettings = BlockSettings {
        hardness: -1.0,
        resistance: 3600000.0,
        friction: 0.6,
        speed_factor: 1.0,
        jump_factor: 1.0,
        requires_correct_tool: true,
        ignited_by_lava: false,
    };
    const LIGHT_EMISSION: PerState<u8> = PerState::Uniform(0);
    const LIGHT_DAMPENING: PerState<u8> = PerState::Uniform(15);
    const FLAGS: PerState<StateFlags> = PerState::Uniform(StateFlags(1054));
    type Behavior = crate::family::Directional<true>;
}
