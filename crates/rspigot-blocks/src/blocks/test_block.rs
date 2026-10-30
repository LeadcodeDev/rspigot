use crate::{Block, BlockId, BlockSettings, PerState, Property, StateFlags, StateId};

pub struct TestBlock;

impl Block for TestBlock {
    const NAME: &'static str = "minecraft:test_block";
    const ID: BlockId = BlockId(997);
    const FIRST_STATE: StateId = StateId(25095);
    const STATE_COUNT: u16 = 4;
    const DEFAULT_STATE: StateId = StateId(25095);
    const PROPERTIES: &'static [Property] = &[
        Property { name: "mode", values: &["start", "log", "fail", "accept"] },
    ];
    const ITEM: Option<u16> = Some(999);
    const WALL_VARIANT: Option<BlockId> = None;
    const SETTINGS: BlockSettings = BlockSettings {
        hardness: -1.0,
        resistance: 3600000.0,
        friction: 0.6,
        speed_factor: 1.0,
        jump_factor: 1.0,
        requires_correct_tool: false,
        ignited_by_lava: false,
    };
    const LIGHT_EMISSION: PerState<u8> = PerState::Uniform(0);
    const LIGHT_DAMPENING: PerState<u8> = PerState::Uniform(15);
    const FLAGS: PerState<StateFlags> = PerState::Uniform(StateFlags(1054));
    type Behavior = crate::family::Plain;
}
