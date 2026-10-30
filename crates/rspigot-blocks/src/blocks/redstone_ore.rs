use crate::{Block, BlockId, BlockSettings, PerState, Property, StateFlags, StateId};

pub struct RedstoneOre;

impl Block for RedstoneOre {
    const NAME: &'static str = "minecraft:redstone_ore";
    const ID: BlockId = BlockId(320);
    const FIRST_STATE: StateId = StateId(8551);
    const STATE_COUNT: u16 = 2;
    const DEFAULT_STATE: StateId = StateId(8552);
    const PROPERTIES: &'static [Property] = &[
        Property { name: "lit", values: &["true", "false"] },
    ];
    const ITEM: Option<u16> = Some(101);
    const WALL_VARIANT: Option<BlockId> = None;
    const SETTINGS: BlockSettings = BlockSettings {
        hardness: 3.0,
        resistance: 3.0,
        friction: 0.6,
        speed_factor: 1.0,
        jump_factor: 1.0,
        requires_correct_tool: true,
        ignited_by_lava: false,
    };
    const LIGHT_EMISSION: PerState<u8> = PerState::Varies(&[
        9, 0,
    ]);
    const LIGHT_DAMPENING: PerState<u8> = PerState::Uniform(15);
    const FLAGS: PerState<StateFlags> = PerState::Varies(&[
        StateFlags(1566), StateFlags(1054),
    ]);
    type Behavior = crate::family::Plain;
}
