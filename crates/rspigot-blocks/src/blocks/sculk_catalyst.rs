use crate::{Block, BlockId, BlockSettings, PerState, Property, StateFlags, StateId};

pub struct SculkCatalyst;

impl Block for SculkCatalyst {
    const NAME: &'static str = "minecraft:sculk_catalyst";
    const ID: BlockId = BlockId(1122);
    const FIRST_STATE: StateId = StateId(31129);
    const STATE_COUNT: u16 = 2;
    const DEFAULT_STATE: StateId = StateId(31130);
    const PROPERTIES: &'static [Property] = &[
        Property { name: "bloom", values: &["true", "false"] },
    ];
    const ITEM: Option<u16> = Some(505);
    const WALL_VARIANT: Option<BlockId> = None;
    const SETTINGS: BlockSettings = BlockSettings {
        hardness: 3.0,
        resistance: 3.0,
        friction: 0.6,
        speed_factor: 1.0,
        jump_factor: 1.0,
        requires_correct_tool: false,
        ignited_by_lava: false,
    };
    const LIGHT_EMISSION: PerState<u8> = PerState::Uniform(6);
    const LIGHT_DAMPENING: PerState<u8> = PerState::Uniform(15);
    const FLAGS: PerState<StateFlags> = PerState::Uniform(StateFlags(1054));
    type Behavior = crate::family::Plain;
}
