use crate::{Block, BlockId, BlockSettings, PerState, Property, StateFlags, StateId};

pub struct FrostedIce;

impl Block for FrostedIce {
    const NAME: &'static str = "minecraft:frosted_ice";
    const ID: BlockId = BlockId(728);
    const FIRST_STATE: StateId = StateId(16822);
    const STATE_COUNT: u16 = 4;
    const DEFAULT_STATE: StateId = StateId(16822);
    const PROPERTIES: &'static [Property] = &[
        Property { name: "age", values: &["0", "1", "2", "3"] },
    ];
    const ITEM: Option<u16> = None;
    const WALL_VARIANT: Option<BlockId> = None;
    const SETTINGS: BlockSettings = BlockSettings {
        hardness: 0.5,
        resistance: 0.5,
        friction: 0.98,
        speed_factor: 1.0,
        jump_factor: 1.0,
        requires_correct_tool: false,
        ignited_by_lava: false,
    };
    const LIGHT_EMISSION: PerState<u8> = PerState::Uniform(0);
    const LIGHT_DAMPENING: PerState<u8> = PerState::Uniform(1);
    const FLAGS: PerState<StateFlags> = PerState::Uniform(StateFlags(1030));
    type Behavior = crate::family::Plain;
}
