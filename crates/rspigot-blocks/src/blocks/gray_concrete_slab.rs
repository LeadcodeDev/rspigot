use crate::{Block, BlockId, BlockSettings, PerState, Property, StateFlags, StateId};

pub struct GrayConcreteSlab;

impl Block for GrayConcreteSlab {
    const NAME: &'static str = "minecraft:gray_concrete_slab";
    const ID: BlockId = BlockId(807);
    const FIRST_STATE: StateId = StateId(18349);
    const STATE_COUNT: u16 = 6;
    const DEFAULT_STATE: StateId = StateId(18352);
    const PROPERTIES: &'static [Property] = &[
        Property { name: "type", values: &["top", "bottom", "double"] },
        Property { name: "waterlogged", values: &["true", "false"] },
    ];
    const ITEM: Option<u16> = Some(728);
    const WALL_VARIANT: Option<BlockId> = None;
    const SETTINGS: BlockSettings = BlockSettings {
        hardness: 1.8,
        resistance: 1.8,
        friction: 0.6,
        speed_factor: 1.0,
        jump_factor: 1.0,
        requires_correct_tool: true,
        ignited_by_lava: false,
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
