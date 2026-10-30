use crate::{Block, BlockId, BlockSettings, PerState, Property, StateFlags, StateId};

pub struct Grindstone;

impl Block for Grindstone {
    const NAME: &'static str = "minecraft:grindstone";
    const ID: BlockId = BlockId(934);
    const FIRST_STATE: StateId = StateId(24129);
    const STATE_COUNT: u16 = 12;
    const DEFAULT_STATE: StateId = StateId(24133);
    const PROPERTIES: &'static [Property] = &[
        Property { name: "face", values: &["floor", "wall", "ceiling"] },
        Property { name: "facing", values: &["north", "south", "west", "east"] },
    ];
    const ITEM: Option<u16> = Some(1511);
    const WALL_VARIANT: Option<BlockId> = None;
    const SETTINGS: BlockSettings = BlockSettings {
        hardness: 2.0,
        resistance: 6.0,
        friction: 0.6,
        speed_factor: 1.0,
        jump_factor: 1.0,
        requires_correct_tool: true,
        ignited_by_lava: false,
    };
    const LIGHT_EMISSION: PerState<u8> = PerState::Uniform(0);
    const LIGHT_DAMPENING: PerState<u8> = PerState::Uniform(0);
    const FLAGS: PerState<StateFlags> = PerState::Uniform(StateFlags(46));
    type Behavior = crate::family::Plain;
}
