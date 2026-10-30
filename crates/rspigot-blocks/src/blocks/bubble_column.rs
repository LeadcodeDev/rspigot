use crate::{Block, BlockId, BlockSettings, PerState, Property, StateFlags, StateId};

pub struct BubbleColumn;

impl Block for BubbleColumn {
    const NAME: &'static str = "minecraft:bubble_column";
    const ID: BlockId = BlockId(886);
    const FIRST_STATE: StateId = StateId(18651);
    const STATE_COUNT: u16 = 2;
    const DEFAULT_STATE: StateId = StateId(18651);
    const PROPERTIES: &'static [Property] = &[
        Property { name: "drag", values: &["true", "false"] },
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
    const LIGHT_EMISSION: PerState<u8> = PerState::Uniform(0);
    const LIGHT_DAMPENING: PerState<u8> = PerState::Uniform(1);
    const FLAGS: PerState<StateFlags> = PerState::Uniform(StateFlags(192));
    type Behavior = crate::family::Plain;
}
