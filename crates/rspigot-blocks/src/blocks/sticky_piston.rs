use crate::{Block, BlockId, BlockSettings, PerState, Property, StateFlags, StateId};

pub struct StickyPiston;

impl Block for StickyPiston {
    const NAME: &'static str = "minecraft:sticky_piston";
    const ID: BlockId = BlockId(138);
    const FIRST_STATE: StateId = StateId(2350);
    const STATE_COUNT: u16 = 12;
    const DEFAULT_STATE: StateId = StateId(2356);
    const PROPERTIES: &'static [Property] = &[
        Property { name: "extended", values: &["true", "false"] },
        Property { name: "facing", values: &["north", "east", "south", "west", "up", "down"] },
    ];
    const ITEM: Option<u16> = Some(830);
    const WALL_VARIANT: Option<BlockId> = None;
    const SETTINGS: BlockSettings = BlockSettings {
        hardness: 1.5,
        resistance: 1.5,
        friction: 0.6,
        speed_factor: 1.0,
        jump_factor: 1.0,
        requires_correct_tool: false,
        ignited_by_lava: false,
    };
    const LIGHT_EMISSION: PerState<u8> = PerState::Uniform(0);
    const LIGHT_DAMPENING: PerState<u8> = PerState::Varies(&[
        0, 0, 0, 0, 0, 0, 15, 15, 15, 15, 15, 15,
    ]);
    const FLAGS: PerState<StateFlags> = PerState::Varies(&[
        StateFlags(46), StateFlags(46), StateFlags(46), StateFlags(46), StateFlags(46),
        StateFlags(46), StateFlags(1054), StateFlags(1054), StateFlags(1054), StateFlags(1054),
        StateFlags(1054), StateFlags(1054),
    ]);
    type Behavior = crate::family::Directional<true>;
}
