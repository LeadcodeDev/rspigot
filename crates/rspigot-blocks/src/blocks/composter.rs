use crate::{Block, BlockId, BlockSettings, PerState, Property, StateFlags, StateId};

pub struct Composter;

impl Block for Composter {
    const NAME: &'static str = "minecraft:composter";
    const ID: BlockId = BlockId(999);
    const FIRST_STATE: StateId = StateId(25100);
    const STATE_COUNT: u16 = 9;
    const DEFAULT_STATE: StateId = StateId(25100);
    const PROPERTIES: &'static [Property] = &[
        Property { name: "level", values: &["0", "1", "2", "3", "4", "5", "6", "7", "8"] },
    ];
    const ITEM: Option<u16> = Some(1505);
    const WALL_VARIANT: Option<BlockId> = None;
    const SETTINGS: BlockSettings = BlockSettings {
        hardness: 0.6,
        resistance: 0.6,
        friction: 0.6,
        speed_factor: 1.0,
        jump_factor: 1.0,
        requires_correct_tool: false,
        ignited_by_lava: true,
    };
    const LIGHT_EMISSION: PerState<u8> = PerState::Uniform(0);
    const LIGHT_DAMPENING: PerState<u8> = PerState::Uniform(0);
    const FLAGS: PerState<StateFlags> = PerState::Uniform(StateFlags(46));
    type Behavior = crate::family::Plain;
}
