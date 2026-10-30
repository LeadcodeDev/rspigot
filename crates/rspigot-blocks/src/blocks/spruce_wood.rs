use crate::{Block, BlockId, BlockSettings, PerState, Property, StateFlags, StateId};

pub struct SpruceWood;

impl Block for SpruceWood {
    const NAME: &'static str = "minecraft:spruce_wood";
    const ID: BlockId = BlockId(76);
    const FIRST_STATE: StateId = StateId(213);
    const STATE_COUNT: u16 = 3;
    const DEFAULT_STATE: StateId = StateId(214);
    const PROPERTIES: &'static [Property] = &[
        Property { name: "axis", values: &["x", "y", "z"] },
    ];
    const ITEM: Option<u16> = Some(204);
    const WALL_VARIANT: Option<BlockId> = None;
    const SETTINGS: BlockSettings = BlockSettings {
        hardness: 2.0,
        resistance: 2.0,
        friction: 0.6,
        speed_factor: 1.0,
        jump_factor: 1.0,
        requires_correct_tool: false,
        ignited_by_lava: true,
    };
    const LIGHT_EMISSION: PerState<u8> = PerState::Uniform(0);
    const LIGHT_DAMPENING: PerState<u8> = PerState::Uniform(15);
    const FLAGS: PerState<StateFlags> = PerState::Uniform(StateFlags(1054));
    type Behavior = crate::family::Pillar;
}
