use crate::{Block, BlockId, BlockSettings, PerState, Property, StateFlags, StateId};

pub struct RedstoneLamp;

impl Block for RedstoneLamp {
    const NAME: &'static str = "minecraft:redstone_lamp";
    const ID: BlockId = BlockId(445);
    const FIRST_STATE: StateId = StateId(11213);
    const STATE_COUNT: u16 = 2;
    const DEFAULT_STATE: StateId = StateId(11214);
    const PROPERTIES: &'static [Property] = &[
        Property { name: "lit", values: &["true", "false"] },
    ];
    const ITEM: Option<u16> = Some(854);
    const WALL_VARIANT: Option<BlockId> = None;
    const SETTINGS: BlockSettings = BlockSettings {
        hardness: 0.3,
        resistance: 0.3,
        friction: 0.6,
        speed_factor: 1.0,
        jump_factor: 1.0,
        requires_correct_tool: false,
        ignited_by_lava: false,
    };
    const LIGHT_EMISSION: PerState<u8> = PerState::Varies(&[
        15, 0,
    ]);
    const LIGHT_DAMPENING: PerState<u8> = PerState::Uniform(15);
    const FLAGS: PerState<StateFlags> = PerState::Uniform(StateFlags(1054));
    type Behavior = crate::family::Plain;
}
