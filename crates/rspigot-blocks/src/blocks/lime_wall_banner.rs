use crate::{Block, BlockId, BlockSettings, PerState, Property, StateFlags, StateId};

pub struct LimeWallBanner;

impl Block for LimeWallBanner {
    const NAME: &'static str = "minecraft:lime_wall_banner";
    const ID: BlockId = BlockId(638);
    const FIRST_STATE: StateId = StateId(15050);
    const STATE_COUNT: u16 = 4;
    const DEFAULT_STATE: StateId = StateId(15050);
    const PROPERTIES: &'static [Property] = &[
        Property { name: "facing", values: &["north", "south", "west", "east"] },
    ];
    const ITEM: Option<u16> = Some(1422);
    const WALL_VARIANT: Option<BlockId> = None;
    const SETTINGS: BlockSettings = BlockSettings {
        hardness: 1.0,
        resistance: 1.0,
        friction: 0.6,
        speed_factor: 1.0,
        jump_factor: 1.0,
        requires_correct_tool: false,
        ignited_by_lava: true,
    };
    const LIGHT_EMISSION: PerState<u8> = PerState::Uniform(0);
    const LIGHT_DAMPENING: PerState<u8> = PerState::Uniform(0);
    const FLAGS: PerState<StateFlags> = PerState::Uniform(StateFlags(34));
    type Behavior = crate::family::Plain;
}
