use crate::{Block, BlockId, BlockSettings, PerState, Property, StateFlags, StateId};

pub struct WeatheredCopperBulb;

impl Block for WeatheredCopperBulb {
    const NAME: &'static str = "minecraft:weathered_copper_bulb";
    const ID: BlockId = BlockId(1192);
    const FIRST_STATE: StateId = StateId(32901);
    const STATE_COUNT: u16 = 4;
    const DEFAULT_STATE: StateId = StateId(32904);
    const PROPERTIES: &'static [Property] = &[
        Property { name: "lit", values: &["true", "false"] },
        Property { name: "powered", values: &["true", "false"] },
    ];
    const ITEM: Option<u16> = Some(1631);
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
    const LIGHT_EMISSION: PerState<u8> = PerState::Varies(&[
        8, 8, 0, 0,
    ]);
    const LIGHT_DAMPENING: PerState<u8> = PerState::Uniform(15);
    const FLAGS: PerState<StateFlags> = PerState::Uniform(StateFlags(1566));
    type Behavior = crate::family::Plain;
}
