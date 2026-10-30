use crate::{Block, BlockId, BlockSettings, PerState, Property, StateFlags, StateId};

pub struct CopperChest;

impl Block for CopperChest {
    const NAME: &'static str = "minecraft:copper_chest";
    const ID: BlockId = BlockId(1198);
    const FIRST_STATE: StateId = StateId(32925);
    const STATE_COUNT: u16 = 24;
    const DEFAULT_STATE: StateId = StateId(32926);
    const PROPERTIES: &'static [Property] = &[
        Property { name: "facing", values: &["north", "south", "west", "east"] },
        Property { name: "type", values: &["single", "left", "right"] },
        Property { name: "waterlogged", values: &["true", "false"] },
    ];
    const ITEM: Option<u16> = Some(1637);
    const WALL_VARIANT: Option<BlockId> = None;
    const SETTINGS: BlockSettings = BlockSettings {
        hardness: 3.0,
        resistance: 6.0,
        friction: 0.6,
        speed_factor: 1.0,
        jump_factor: 1.0,
        requires_correct_tool: true,
        ignited_by_lava: false,
    };
    const LIGHT_EMISSION: PerState<u8> = PerState::Uniform(0);
    const LIGHT_DAMPENING: PerState<u8> = PerState::Varies(&[
        1, 0, 1, 0, 1, 0, 1, 0, 1, 0, 1, 0, 1, 0, 1, 0, 1, 0, 1, 0, 1, 0, 1, 0,
    ]);
    const FLAGS: PerState<StateFlags> = PerState::Varies(&[
        StateFlags(654), StateFlags(558), StateFlags(654), StateFlags(558), StateFlags(654),
        StateFlags(558), StateFlags(654), StateFlags(558), StateFlags(654), StateFlags(558),
        StateFlags(654), StateFlags(558), StateFlags(654), StateFlags(558), StateFlags(654),
        StateFlags(558), StateFlags(654), StateFlags(558), StateFlags(654), StateFlags(558),
        StateFlags(654), StateFlags(558), StateFlags(654), StateFlags(558),
    ]);
    type Behavior = crate::family::HorizontalAway;
}
