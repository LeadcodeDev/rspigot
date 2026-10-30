use crate::{Block, BlockId, BlockSettings, PerState, Property, StateFlags, StateId};

pub struct WaxedOxidizedCopperChain;

impl Block for WaxedOxidizedCopperChain {
    const NAME: &'static str = "minecraft:waxed_oxidized_copper_chain";
    const ID: BlockId = BlockId(408);
    const FIRST_STATE: StateId = StateId(10028);
    const STATE_COUNT: u16 = 6;
    const DEFAULT_STATE: StateId = StateId(10031);
    const PROPERTIES: &'static [Property] = &[
        Property { name: "axis", values: &["x", "y", "z"] },
        Property { name: "waterlogged", values: &["true", "false"] },
    ];
    const ITEM: Option<u16> = Some(481);
    const WALL_VARIANT: Option<BlockId> = None;
    const SETTINGS: BlockSettings = BlockSettings {
        hardness: 5.0,
        resistance: 6.0,
        friction: 0.6,
        speed_factor: 1.0,
        jump_factor: 1.0,
        requires_correct_tool: true,
        ignited_by_lava: false,
    };
    const LIGHT_EMISSION: PerState<u8> = PerState::Uniform(0);
    const LIGHT_DAMPENING: PerState<u8> = PerState::Varies(&[
        1, 0, 1, 0, 1, 0,
    ]);
    const FLAGS: PerState<StateFlags> = PerState::Varies(&[
        StateFlags(134), StateFlags(38), StateFlags(134), StateFlags(38), StateFlags(134),
        StateFlags(38),
    ]);
    type Behavior = crate::family::Pillar;
}
