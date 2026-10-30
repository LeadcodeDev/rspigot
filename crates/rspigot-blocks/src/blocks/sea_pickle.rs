use crate::{Block, BlockId, BlockSettings, PerState, Property, StateFlags, StateId};

pub struct SeaPickle;

impl Block for SeaPickle {
    const NAME: &'static str = "minecraft:sea_pickle";
    const ID: BlockId = BlockId(878);
    const FIRST_STATE: StateId = StateId(18624);
    const STATE_COUNT: u16 = 8;
    const DEFAULT_STATE: StateId = StateId(18624);
    const PROPERTIES: &'static [Property] = &[
        Property { name: "pickles", values: &["1", "2", "3", "4"] },
        Property { name: "waterlogged", values: &["true", "false"] },
    ];
    const ITEM: Option<u16> = Some(249);
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
    const LIGHT_EMISSION: PerState<u8> = PerState::Varies(&[
        6, 0, 9, 0, 12, 0, 15, 0,
    ]);
    const LIGHT_DAMPENING: PerState<u8> = PerState::Varies(&[
        1, 0, 1, 0, 1, 0, 1, 0,
    ]);
    const FLAGS: PerState<StateFlags> = PerState::Varies(&[
        StateFlags(132), StateFlags(36), StateFlags(132), StateFlags(36), StateFlags(132),
        StateFlags(36), StateFlags(132), StateFlags(36),
    ]);
    type Behavior = crate::family::Plain;
}
