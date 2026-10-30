use crate::{Block, BlockId, BlockSettings, PerState, Property, StateFlags, StateId};

pub struct AcaciaFenceGate;

impl Block for AcaciaFenceGate {
    const NAME: &'static str = "minecraft:acacia_fence_gate";
    const ID: BlockId = BlockId(685);
    const FIRST_STATE: StateId = StateId(15427);
    const STATE_COUNT: u16 = 32;
    const DEFAULT_STATE: StateId = StateId(15434);
    const PROPERTIES: &'static [Property] = &[
        Property { name: "facing", values: &["north", "south", "west", "east"] },
        Property { name: "in_wall", values: &["true", "false"] },
        Property { name: "open", values: &["true", "false"] },
        Property { name: "powered", values: &["true", "false"] },
    ];
    const ITEM: Option<u16> = Some(936);
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
