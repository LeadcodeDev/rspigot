use crate::{Block, BlockId, BlockSettings, PerState, Property, StateFlags, StateId};

pub struct GrayShulkerBox;

impl Block for GrayShulkerBox {
    const NAME: &'static str = "minecraft:gray_shulker_box";
    const ID: BlockId = BlockId(743);
    const FIRST_STATE: StateId = StateId(16893);
    const STATE_COUNT: u16 = 6;
    const DEFAULT_STATE: StateId = StateId(16897);
    const PROPERTIES: &'static [Property] = &[
        Property { name: "facing", values: &["north", "east", "south", "west", "up", "down"] },
    ];
    const ITEM: Option<u16> = Some(664);
    const WALL_VARIANT: Option<BlockId> = None;
    const SETTINGS: BlockSettings = BlockSettings {
        hardness: 2.0,
        resistance: 2.0,
        friction: 0.6,
        speed_factor: 1.0,
        jump_factor: 1.0,
        requires_correct_tool: false,
        ignited_by_lava: false,
    };
    const LIGHT_EMISSION: PerState<u8> = PerState::Uniform(0);
    const LIGHT_DAMPENING: PerState<u8> = PerState::Uniform(1);
    const FLAGS: PerState<StateFlags> = PerState::Uniform(StateFlags(1030));
    type Behavior = crate::family::FaceAttached;
}
