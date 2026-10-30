use crate::{Block, BlockId, BlockSettings, PerState, Property, StateFlags, StateId};

pub struct CopperGrate;

impl Block for CopperGrate {
    const NAME: &'static str = "minecraft:copper_grate";
    const ID: BlockId = BlockId(1182);
    const FIRST_STATE: StateId = StateId(32877);
    const STATE_COUNT: u16 = 2;
    const DEFAULT_STATE: StateId = StateId(32878);
    const PROPERTIES: &'static [Property] = &[
        Property { name: "waterlogged", values: &["true", "false"] },
    ];
    const ITEM: Option<u16> = Some(1621);
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
    const LIGHT_DAMPENING: PerState<u8> = PerState::Uniform(0);
    const FLAGS: PerState<StateFlags> = PerState::Varies(&[
        StateFlags(1702), StateFlags(1574),
    ]);
    type Behavior = crate::family::Plain;
}
