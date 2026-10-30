use crate::{Block, BlockId, BlockSettings, PerState, Property, StateFlags, StateId};

pub struct Ladder;

impl Block for Ladder {
    const NAME: &'static str = "minecraft:ladder";
    const ID: BlockId = BlockId(266);
    const FIRST_STATE: StateId = StateId(7307);
    const STATE_COUNT: u16 = 8;
    const DEFAULT_STATE: StateId = StateId(7308);
    const PROPERTIES: &'static [Property] = &[
        Property { name: "facing", values: &["north", "south", "west", "east"] },
        Property { name: "waterlogged", values: &["true", "false"] },
    ];
    const ITEM: Option<u16> = Some(408);
    const WALL_VARIANT: Option<BlockId> = None;
    const SETTINGS: BlockSettings = BlockSettings {
        hardness: 0.4,
        resistance: 0.4,
        friction: 0.6,
        speed_factor: 1.0,
        jump_factor: 1.0,
        requires_correct_tool: false,
        ignited_by_lava: false,
    };
    const LIGHT_EMISSION: PerState<u8> = PerState::Uniform(0);
    const LIGHT_DAMPENING: PerState<u8> = PerState::Varies(&[
        1, 0, 1, 0, 1, 0, 1, 0,
    ]);
    const FLAGS: PerState<StateFlags> = PerState::Varies(&[
        StateFlags(132), StateFlags(36), StateFlags(132), StateFlags(36), StateFlags(132),
        StateFlags(36), StateFlags(132), StateFlags(36),
    ]);
    type Behavior = crate::family::FaceAttached;
}
