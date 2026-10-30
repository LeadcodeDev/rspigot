use crate::{Block, BlockId, BlockSettings, PerState, Property, StateFlags, StateId};

pub struct LeafLitter;

impl Block for LeafLitter {
    const NAME: &'static str = "minecraft:leaf_litter";
    const ID: BlockId = BlockId(1233);
    const FIRST_STATE: StateId = StateId(33696);
    const STATE_COUNT: u16 = 16;
    const DEFAULT_STATE: StateId = StateId(33696);
    const PROPERTIES: &'static [Property] = &[
        Property { name: "facing", values: &["north", "south", "west", "east"] },
        Property { name: "segment_amount", values: &["1", "2", "3", "4"] },
    ];
    const ITEM: Option<u16> = Some(331);
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
    const LIGHT_EMISSION: PerState<u8> = PerState::Uniform(0);
    const LIGHT_DAMPENING: PerState<u8> = PerState::Uniform(0);
    const FLAGS: PerState<StateFlags> = PerState::Uniform(StateFlags(96));
    type Behavior = crate::family::HorizontalAway;
}
