use crate::{Block, BlockId, BlockSettings, PerState, Property, StateFlags, StateId};

pub struct CherryTrapdoor;

impl Block for CherryTrapdoor {
    const NAME: &'static str = "minecraft:cherry_trapdoor";
    const ID: BlockId = BlockId(370);
    const FIRST_STATE: StateId = StateId(9104);
    const STATE_COUNT: u16 = 64;
    const DEFAULT_STATE: StateId = StateId(9119);
    const PROPERTIES: &'static [Property] = &[
        Property { name: "facing", values: &["north", "south", "west", "east"] },
        Property { name: "half", values: &["top", "bottom"] },
        Property { name: "open", values: &["true", "false"] },
        Property { name: "powered", values: &["true", "false"] },
        Property { name: "waterlogged", values: &["true", "false"] },
    ];
    const ITEM: Option<u16> = Some(916);
    const WALL_VARIANT: Option<BlockId> = None;
    const SETTINGS: BlockSettings = BlockSettings {
        hardness: 3.0,
        resistance: 3.0,
        friction: 0.6,
        speed_factor: 1.0,
        jump_factor: 1.0,
        requires_correct_tool: false,
        ignited_by_lava: true,
    };
    const LIGHT_EMISSION: PerState<u8> = PerState::Uniform(0);
    const LIGHT_DAMPENING: PerState<u8> = PerState::Varies(&[
        1, 0, 1, 0, 1, 0, 1, 0, 1, 0, 1, 0, 1, 0, 1, 0, 1, 0, 1, 0, 1, 0, 1, 0, 1, 0, 1, 0, 1, 0, 1,
        0, 1, 0, 1, 0, 1, 0, 1, 0, 1, 0, 1, 0, 1, 0, 1, 0, 1, 0, 1, 0, 1, 0, 1, 0, 1, 0, 1, 0, 1, 0,
        1, 0,
    ]);
    const FLAGS: PerState<StateFlags> = PerState::Varies(&[
        StateFlags(134), StateFlags(38), StateFlags(134), StateFlags(38), StateFlags(134),
        StateFlags(38), StateFlags(134), StateFlags(38), StateFlags(134), StateFlags(38),
        StateFlags(134), StateFlags(38), StateFlags(134), StateFlags(38), StateFlags(134),
        StateFlags(38), StateFlags(134), StateFlags(38), StateFlags(134), StateFlags(38),
        StateFlags(134), StateFlags(38), StateFlags(134), StateFlags(38), StateFlags(134),
        StateFlags(38), StateFlags(134), StateFlags(38), StateFlags(134), StateFlags(38),
        StateFlags(134), StateFlags(38), StateFlags(134), StateFlags(38), StateFlags(134),
        StateFlags(38), StateFlags(134), StateFlags(38), StateFlags(134), StateFlags(38),
        StateFlags(134), StateFlags(38), StateFlags(134), StateFlags(38), StateFlags(134),
        StateFlags(38), StateFlags(134), StateFlags(38), StateFlags(134), StateFlags(38),
        StateFlags(134), StateFlags(38), StateFlags(134), StateFlags(38), StateFlags(134),
        StateFlags(38), StateFlags(134), StateFlags(38), StateFlags(134), StateFlags(38),
        StateFlags(134), StateFlags(38), StateFlags(134), StateFlags(38),
    ]);
    type Behavior = crate::family::Trapdoor<true>;
}
