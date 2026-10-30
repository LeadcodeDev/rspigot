use crate::{Block, BlockId, BlockSettings, PerState, Property, StateFlags, StateId};

pub struct OakFenceGate;

impl Block for OakFenceGate {
    const NAME: &'static str = "minecraft:oak_fence_gate";
    const ID: BlockId = BlockId(419);
    const FIRST_STATE: StateId = StateId(10380);
    const STATE_COUNT: u16 = 32;
    const DEFAULT_STATE: StateId = StateId(10387);
    const PROPERTIES: &'static [Property] = &[
        Property { name: "facing", values: &["north", "south", "west", "east"] },
        Property { name: "in_wall", values: &["true", "false"] },
        Property { name: "open", values: &["true", "false"] },
        Property { name: "powered", values: &["true", "false"] },
    ];
    const ITEM: Option<u16> = Some(932);
    const WALL_VARIANT: Option<BlockId> = None;
    const SETTINGS: BlockSettings = BlockSettings {
        hardness: 2.0,
        resistance: 3.0,
        friction: 0.6,
        speed_factor: 1.0,
        jump_factor: 1.0,
        requires_correct_tool: false,
        ignited_by_lava: true,
    };
    const LIGHT_EMISSION: PerState<u8> = PerState::Uniform(0);
    const LIGHT_DAMPENING: PerState<u8> = PerState::Uniform(0);
    const FLAGS: PerState<StateFlags> = PerState::Varies(&[
        StateFlags(42), StateFlags(42), StateFlags(46), StateFlags(46), StateFlags(42),
        StateFlags(42), StateFlags(46), StateFlags(46), StateFlags(42), StateFlags(42),
        StateFlags(46), StateFlags(46), StateFlags(42), StateFlags(42), StateFlags(46),
        StateFlags(46), StateFlags(42), StateFlags(42), StateFlags(46), StateFlags(46),
        StateFlags(42), StateFlags(42), StateFlags(46), StateFlags(46), StateFlags(42),
        StateFlags(42), StateFlags(46), StateFlags(46), StateFlags(42), StateFlags(42),
        StateFlags(46), StateFlags(46),
    ]);
    type Behavior = crate::family::FenceGate;
}
