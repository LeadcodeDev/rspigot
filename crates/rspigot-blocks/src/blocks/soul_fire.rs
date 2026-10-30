use crate::{Block, BlockId, BlockSettings, PerState, Property, StateFlags, StateId};

pub struct SoulFire;

impl Block for SoulFire {
    const NAME: &'static str = "minecraft:soul_fire";
    const ID: BlockId = BlockId(241);
    const FIRST_STATE: StateId = StateId(5443);
    const STATE_COUNT: u16 = 1;
    const DEFAULT_STATE: StateId = StateId(5443);
    const PROPERTIES: &'static [Property] = &[];
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
    const LIGHT_EMISSION: PerState<u8> = PerState::Uniform(10);
    const LIGHT_DAMPENING: PerState<u8> = PerState::Uniform(0);
    const FLAGS: PerState<StateFlags> = PerState::Uniform(StateFlags(96));
    type Behavior = crate::family::Plain;
}
