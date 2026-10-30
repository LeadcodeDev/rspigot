use crate::{Block, BlockId, BlockSettings, PerState, Property, StateFlags, StateId};

pub struct PaleOakSlab;

impl Block for PaleOakSlab {
    const NAME: &'static str = "minecraft:pale_oak_slab";
    const ID: BlockId = BlockId(660);
    const FIRST_STATE: StateId = StateId(15219);
    const STATE_COUNT: u16 = 6;
    const DEFAULT_STATE: StateId = StateId(15222);
    const PROPERTIES: &'static [Property] = &[
        Property { name: "type", values: &["top", "bottom", "double"] },
        Property { name: "waterlogged", values: &["true", "false"] },
    ];
    const ITEM: Option<u16> = Some(348);
    const WALL_VARIANT: Option<BlockId> = None;
    const SETTINGS: BlockSettings = BlockSettings {
        hardness: 2.0,
        resistance: 3.0,
        friction: 0.6,
        speed_factor: 1.0,
        jump_factor: 1.0,
        requires_correct_tool: false,
        ignited_by_lava: true,
    };
    const LIGHT_EMISSION: PerState<u8> = PerState::Uniform(0);
    const LIGHT_DAMPENING: PerState<u8> = PerState::Varies(&[
        1, 0, 1, 0, 15, 15,
    ]);
    const FLAGS: PerState<StateFlags> = PerState::Varies(&[
        StateFlags(142), StateFlags(46), StateFlags(142), StateFlags(46), StateFlags(1182),
        StateFlags(1054),
    ]);
    type Behavior = crate::family::Slab;
}
