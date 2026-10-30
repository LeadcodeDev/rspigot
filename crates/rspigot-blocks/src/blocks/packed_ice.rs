use crate::{Block, BlockId, BlockSettings, PerState, Property, StateFlags, StateId};

pub struct PackedIce;

impl Block for PackedIce {
    const NAME: &'static str = "minecraft:packed_ice";
    const ID: BlockId = BlockId(610);
    const FIRST_STATE: StateId = StateId(14761);
    const STATE_COUNT: u16 = 1;
    const DEFAULT_STATE: StateId = StateId(14761);
    const PROPERTIES: &'static [Property] = &[];
    const ITEM: Option<u16> = Some(597);
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
    const LIGHT_DAMPENING: PerState<u8> = PerState::Uniform(15);
    const FLAGS: PerState<StateFlags> = PerState::Uniform(StateFlags(1054));
    type Behavior = crate::family::Plain;
}
