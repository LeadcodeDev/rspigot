use crate::{Block, BlockId, BlockSettings, PerState, Property, StateFlags, StateId};

pub struct EnderChest;

impl Block for EnderChest {
    const NAME: &'static str = "minecraft:ender_chest";
    const ID: BlockId = BlockId(451);
    const FIRST_STATE: StateId = StateId(11317);
    const STATE_COUNT: u16 = 8;
    const DEFAULT_STATE: StateId = StateId(11318);
    const PROPERTIES: &'static [Property] = &[
        Property { name: "facing", values: &["north", "south", "west", "east"] },
        Property { name: "waterlogged", values: &["true", "false"] },
    ];
    const ITEM: Option<u16> = Some(513);
    const WALL_VARIANT: Option<BlockId> = None;
    const SETTINGS: BlockSettings = BlockSettings {
        hardness: 22.5,
        resistance: 600.0,
        friction: 0.6,
        speed_factor: 1.0,
        jump_factor: 1.0,
        requires_correct_tool: false,
        ignited_by_lava: false,
    };
    const LIGHT_EMISSION: PerState<u8> = PerState::Uniform(7);
    const LIGHT_DAMPENING: PerState<u8> = PerState::Varies(&[
        1, 0, 1, 0, 1, 0, 1, 0,
    ]);
    const FLAGS: PerState<StateFlags> = PerState::Varies(&[
        StateFlags(142), StateFlags(46), StateFlags(142), StateFlags(46), StateFlags(142),
        StateFlags(46), StateFlags(142), StateFlags(46),
    ]);
    type Behavior = crate::family::HorizontalAway;
}
