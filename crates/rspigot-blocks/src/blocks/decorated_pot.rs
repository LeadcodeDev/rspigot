use crate::{Block, BlockId, BlockSettings, PerState, Property, StateFlags, StateId};

pub struct DecoratedPot;

impl Block for DecoratedPot {
    const NAME: &'static str = "minecraft:decorated_pot";
    const ID: BlockId = BlockId(1273);
    const FIRST_STATE: StateId = StateId(35443);
    const STATE_COUNT: u16 = 16;
    const DEFAULT_STATE: StateId = StateId(35452);
    const PROPERTIES: &'static [Property] = &[
        Property { name: "cracked", values: &["true", "false"] },
        Property { name: "facing", values: &["north", "south", "west", "east"] },
        Property { name: "waterlogged", values: &["true", "false"] },
    ];
    const ITEM: Option<u16> = Some(392);
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
    const LIGHT_DAMPENING: PerState<u8> = PerState::Varies(&[
        1, 0, 1, 0, 1, 0, 1, 0, 1, 0, 1, 0, 1, 0, 1, 0,
    ]);
    const FLAGS: PerState<StateFlags> = PerState::Varies(&[
        StateFlags(134), StateFlags(38), StateFlags(134), StateFlags(38), StateFlags(134),
        StateFlags(38), StateFlags(134), StateFlags(38), StateFlags(134), StateFlags(38),
        StateFlags(134), StateFlags(38), StateFlags(134), StateFlags(38), StateFlags(134),
        StateFlags(38),
    ]);
    type Behavior = crate::family::HorizontalToward;
}
