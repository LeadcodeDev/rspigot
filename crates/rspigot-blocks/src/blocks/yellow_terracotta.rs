use crate::{Block, BlockId, BlockSettings, PerState, Property, StateFlags, StateId};

pub struct YellowTerracotta;

impl Block for YellowTerracotta {
    const NAME: &'static str = "minecraft:yellow_terracotta";
    const ID: BlockId = BlockId(541);
    const FIRST_STATE: StateId = StateId(13215);
    const STATE_COUNT: u16 = 1;
    const DEFAULT_STATE: StateId = StateId(13215);
    const PROPERTIES: &'static [Property] = &[];
    const ITEM: Option<u16> = Some(565);
    const WALL_VARIANT: Option<BlockId> = None;
    const SETTINGS: BlockSettings = BlockSettings {
        hardness: 1.25,
        resistance: 4.2,
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
