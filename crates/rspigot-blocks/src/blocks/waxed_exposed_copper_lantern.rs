use crate::{Block, BlockId, BlockSettings, PerState, Property, StateFlags, StateId};

pub struct WaxedExposedCopperLantern;

impl Block for WaxedExposedCopperLantern {
    const NAME: &'static str = "minecraft:waxed_exposed_copper_lantern";
    const ID: BlockId = BlockId(946);
    const FIRST_STATE: StateId = StateId(24222);
    const STATE_COUNT: u16 = 4;
    const DEFAULT_STATE: StateId = StateId(24225);
    const PROPERTIES: &'static [Property] = &[
        Property { name: "hanging", values: &["true", "false"] },
        Property { name: "waterlogged", values: &["true", "false"] },
    ];
    const ITEM: Option<u16> = Some(1522);
    const WALL_VARIANT: Option<BlockId> = None;
    const SETTINGS: BlockSettings = BlockSettings {
        hardness: 3.5,
        resistance: 3.5,
        friction: 0.6,
        speed_factor: 1.0,
        jump_factor: 1.0,
        requires_correct_tool: false,
        ignited_by_lava: false,
    };
    const LIGHT_EMISSION: PerState<u8> = PerState::Uniform(15);
    const LIGHT_DAMPENING: PerState<u8> = PerState::Varies(&[
        1, 0, 1, 0,
    ]);
    const FLAGS: PerState<StateFlags> = PerState::Varies(&[
        StateFlags(134), StateFlags(38), StateFlags(134), StateFlags(38),
    ]);
    type Behavior = crate::family::Plain;
}
