use crate::{Block, BlockId, BlockSettings, PerState, Property, StateFlags, StateId};

pub struct CaveVinesPlant;

impl Block for CaveVinesPlant {
    const NAME: &'static str = "minecraft:cave_vines_plant";
    const ID: BlockId = BlockId(1226);
    const FIRST_STATE: StateId = StateId(33658);
    const STATE_COUNT: u16 = 2;
    const DEFAULT_STATE: StateId = StateId(33659);
    const PROPERTIES: &'static [Property] = &[
        Property { name: "berries", values: &["true", "false"] },
    ];
    const ITEM: Option<u16> = None;
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
    const LIGHT_EMISSION: PerState<u8> = PerState::Varies(&[
        14, 0,
    ]);
    const LIGHT_DAMPENING: PerState<u8> = PerState::Uniform(0);
    const FLAGS: PerState<StateFlags> = PerState::Uniform(StateFlags(32));
    type Behavior = crate::family::Plain;
}
