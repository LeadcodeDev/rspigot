use crate::{Block, BlockId, BlockSettings, PerState, Property, StateFlags, StateId};

pub struct Tripwire;

impl Block for Tripwire {
    const NAME: &'static str = "minecraft:tripwire";
    const ID: BlockId = BlockId(453);
    const FIRST_STATE: StateId = StateId(11341);
    const STATE_COUNT: u16 = 128;
    const DEFAULT_STATE: StateId = StateId(11468);
    const PROPERTIES: &'static [Property] = &[
        Property { name: "attached", values: &["true", "false"] },
        Property { name: "disarmed", values: &["true", "false"] },
        Property { name: "east", values: &["true", "false"] },
        Property { name: "north", values: &["true", "false"] },
        Property { name: "powered", values: &["true", "false"] },
        Property { name: "south", values: &["true", "false"] },
        Property { name: "west", values: &["true", "false"] },
    ];
    const ITEM: Option<u16> = Some(1062);
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
    const FLAGS: PerState<StateFlags> = PerState::Uniform(StateFlags(32));
    type Behavior = crate::family::Plain;
}
