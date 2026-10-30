use crate::{Block, BlockId, BlockSettings, PerState, Property, StateFlags, StateId};

pub struct TorchflowerCrop;

impl Block for TorchflowerCrop {
    const NAME: &'static str = "minecraft:torchflower_crop";
    const ID: BlockId = BlockId(720);
    const FIRST_STATE: StateId = StateId(16778);
    const STATE_COUNT: u16 = 2;
    const DEFAULT_STATE: StateId = StateId(16778);
    const PROPERTIES: &'static [Property] = &[
        Property { name: "age", values: &["0", "1"] },
    ];
    const ITEM: Option<u16> = Some(1436);
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
    const FLAGS: PerState<StateFlags> = PerState::Uniform(StateFlags(544));
    type Behavior = crate::family::Plain;
}
