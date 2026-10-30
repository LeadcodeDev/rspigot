use crate::{Block, BlockId, BlockSettings, PerState, Property, StateFlags, StateId};

pub struct Cocoa;

impl Block for Cocoa {
    const NAME: &'static str = "minecraft:cocoa";
    const ID: BlockId = BlockId(446);
    const FIRST_STATE: StateId = StateId(11215);
    const STATE_COUNT: u16 = 12;
    const DEFAULT_STATE: StateId = StateId(11215);
    const PROPERTIES: &'static [Property] = &[
        Property { name: "age", values: &["0", "1", "2"] },
        Property { name: "facing", values: &["north", "south", "west", "east"] },
    ];
    const ITEM: Option<u16> = Some(1198);
    const WALL_VARIANT: Option<BlockId> = None;
    const SETTINGS: BlockSettings = BlockSettings {
        hardness: 0.2,
        resistance: 3.0,
        friction: 0.6,
        speed_factor: 1.0,
        jump_factor: 1.0,
        requires_correct_tool: false,
        ignited_by_lava: false,
    };
    const LIGHT_EMISSION: PerState<u8> = PerState::Uniform(0);
    const LIGHT_DAMPENING: PerState<u8> = PerState::Uniform(0);
    const FLAGS: PerState<StateFlags> = PerState::Varies(&[
        StateFlags(548), StateFlags(548), StateFlags(548), StateFlags(548), StateFlags(548),
        StateFlags(548), StateFlags(548), StateFlags(548), StateFlags(36), StateFlags(36),
        StateFlags(36), StateFlags(36),
    ]);
    type Behavior = crate::family::Plain;
}
