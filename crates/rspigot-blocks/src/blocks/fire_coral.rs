use crate::{Block, BlockId, BlockSettings, PerState, Property, StateFlags, StateId};

pub struct FireCoral;

impl Block for FireCoral {
    const NAME: &'static str = "minecraft:fire_coral";
    const ID: BlockId = BlockId(856);
    const FIRST_STATE: StateId = StateId(18520);
    const STATE_COUNT: u16 = 2;
    const DEFAULT_STATE: StateId = StateId(18520);
    const PROPERTIES: &'static [Property] = &[
        Property { name: "waterlogged", values: &["true", "false"] },
    ];
    const ITEM: Option<u16> = Some(769);
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
        1, 0,
    ]);
    const FLAGS: PerState<StateFlags> = PerState::Varies(&[
        StateFlags(128), StateFlags(32),
    ]);
    type Behavior = crate::family::Plain;
}
