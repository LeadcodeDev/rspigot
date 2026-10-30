use crate::{Block, BlockId, BlockSettings, PerState, Property, StateFlags, StateId};

pub struct YellowWoolSlab;

impl Block for YellowWoolSlab {
    const NAME: &'static str = "minecraft:yellow_wool_slab";
    const ID: BlockId = BlockId(187);
    const FIRST_STATE: StateId = StateId(3729);
    const STATE_COUNT: u16 = 6;
    const DEFAULT_STATE: StateId = StateId(3732);
    const PROPERTIES: &'static [Property] = &[
        Property { name: "type", values: &["top", "bottom", "double"] },
        Property { name: "waterlogged", values: &["true", "false"] },
    ];
    const ITEM: Option<u16> = Some(286);
    const WALL_VARIANT: Option<BlockId> = None;
    const SETTINGS: BlockSettings = BlockSettings {
        hardness: 0.8,
        resistance: 0.8,
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
