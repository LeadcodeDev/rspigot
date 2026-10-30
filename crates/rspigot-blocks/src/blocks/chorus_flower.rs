use crate::{Block, BlockId, BlockSettings, PerState, Property, StateFlags, StateId};

pub struct ChorusFlower;

impl Block for ChorusFlower {
    const NAME: &'static str = "minecraft:chorus_flower";
    const ID: BlockId = BlockId(714);
    const FIRST_STATE: StateId = StateId(16681);
    const STATE_COUNT: u16 = 6;
    const DEFAULT_STATE: StateId = StateId(16681);
    const PROPERTIES: &'static [Property] = &[
        Property { name: "age", values: &["0", "1", "2", "3", "4", "5"] },
    ];
    const ITEM: Option<u16> = Some(398);
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
    const LIGHT_DAMPENING: PerState<u8> = PerState::Uniform(1);
    const FLAGS: PerState<StateFlags> = PerState::Varies(&[
        StateFlags(1540), StateFlags(1540), StateFlags(1540), StateFlags(1540), StateFlags(1540),
        StateFlags(1028),
    ]);
    type Behavior = crate::family::Plain;
}
