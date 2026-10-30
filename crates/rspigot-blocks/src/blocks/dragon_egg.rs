use crate::{Block, BlockId, BlockSettings, PerState, Property, StateFlags, StateId};

pub struct DragonEgg;

impl Block for DragonEgg {
    const NAME: &'static str = "minecraft:dragon_egg";
    const ID: BlockId = BlockId(444);
    const FIRST_STATE: StateId = StateId(11212);
    const STATE_COUNT: u16 = 1;
    const DEFAULT_STATE: StateId = StateId(11212);
    const PROPERTIES: &'static [Property] = &[];
    const ITEM: Option<u16> = Some(511);
    const WALL_VARIANT: Option<BlockId> = None;
    const SETTINGS: BlockSettings = BlockSettings {
        hardness: 3.0,
        resistance: 9.0,
        friction: 0.6,
        speed_factor: 1.0,
        jump_factor: 1.0,
        requires_correct_tool: false,
        ignited_by_lava: false,
    };
    const LIGHT_EMISSION: PerState<u8> = PerState::Uniform(1);
    const LIGHT_DAMPENING: PerState<u8> = PerState::Uniform(0);
    const FLAGS: PerState<StateFlags> = PerState::Uniform(StateFlags(38));
    type Behavior = crate::family::Plain;
}
