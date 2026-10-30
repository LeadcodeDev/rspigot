use crate::{Block, BlockId, BlockSettings, PerState, Property, StateFlags, StateId};

pub struct PotentSulfur;

impl Block for PotentSulfur {
    const NAME: &'static str = "minecraft:potent_sulfur";
    const ID: BlockId = BlockId(1089);
    const FIRST_STATE: StateId = StateId(28045);
    const STATE_COUNT: u16 = 5;
    const DEFAULT_STATE: StateId = StateId(28045);
    const PROPERTIES: &'static [Property] = &[
        Property {
            name: "potent_sulfur_state",
            values: &[
                "dry", "wet", "dormant", "erupting", "continuous",
            ],
        },
    ];
    const ITEM: Option<u16> = Some(27);
    const WALL_VARIANT: Option<BlockId> = None;
    const SETTINGS: BlockSettings = BlockSettings {
        hardness: 1.5,
        resistance: 6.0,
        friction: 0.6,
        speed_factor: 1.0,
        jump_factor: 1.0,
        requires_correct_tool: true,
        ignited_by_lava: false,
    };
    const LIGHT_EMISSION: PerState<u8> = PerState::Uniform(0);
    const LIGHT_DAMPENING: PerState<u8> = PerState::Uniform(15);
    const FLAGS: PerState<StateFlags> = PerState::Uniform(StateFlags(1054));
    type Behavior = crate::family::Plain;
}
