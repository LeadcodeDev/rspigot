use crate::{Block, BlockId, BlockSettings, PerState, Property, StateFlags, StateId};

pub struct PlayerWallHead;

impl Block for PlayerWallHead {
    const NAME: &'static str = "minecraft:player_wall_head";
    const ID: BlockId = BlockId(513);
    const FIRST_STATE: StateId = StateId(12834);
    const STATE_COUNT: u16 = 8;
    const DEFAULT_STATE: StateId = StateId(12835);
    const PROPERTIES: &'static [Property] = &[
        Property { name: "facing", values: &["north", "south", "west", "east"] },
        Property { name: "powered", values: &["true", "false"] },
    ];
    const ITEM: Option<u16> = Some(1386);
    const WALL_VARIANT: Option<BlockId> = None;
    const SETTINGS: BlockSettings = BlockSettings {
        hardness: 1.0,
        resistance: 1.0,
        friction: 0.6,
        speed_factor: 1.0,
        jump_factor: 1.0,
        requires_correct_tool: false,
        ignited_by_lava: false,
    };
    const LIGHT_EMISSION: PerState<u8> = PerState::Uniform(0);
    const LIGHT_DAMPENING: PerState<u8> = PerState::Uniform(0);
    const FLAGS: PerState<StateFlags> = PerState::Uniform(StateFlags(44));
    type Behavior = crate::family::Plain;
}
