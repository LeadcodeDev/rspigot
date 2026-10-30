use crate::{Block, BlockId, BlockSettings, PerState, Property, StateFlags, StateId};

pub struct PaleHangingMoss;

impl Block for PaleHangingMoss {
    const NAME: &'static str = "minecraft:pale_hanging_moss";
    const ID: BlockId = BlockId(1280);
    const FIRST_STATE: StateId = StateId(35716);
    const STATE_COUNT: u16 = 2;
    const DEFAULT_STATE: StateId = StateId(35716);
    const PROPERTIES: &'static [Property] = &[
        Property { name: "tip", values: &["true", "false"] },
    ];
    const ITEM: Option<u16> = Some(335);
    const WALL_VARIANT: Option<BlockId> = None;
    const SETTINGS: BlockSettings = BlockSettings {
        hardness: 0.0,
        resistance: 0.0,
        friction: 0.6,
        speed_factor: 1.0,
        jump_factor: 1.0,
        requires_correct_tool: false,
        ignited_by_lava: true,
    };
    const LIGHT_EMISSION: PerState<u8> = PerState::Uniform(0);
    const LIGHT_DAMPENING: PerState<u8> = PerState::Uniform(0);
    const FLAGS: PerState<StateFlags> = PerState::Uniform(StateFlags(32));
    type Behavior = crate::family::Plain;
}
