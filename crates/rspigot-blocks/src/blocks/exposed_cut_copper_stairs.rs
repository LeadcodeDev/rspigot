use crate::{Block, BlockId, BlockSettings, PerState, Property, StateFlags, StateId};

pub struct ExposedCutCopperStairs;

impl Block for ExposedCutCopperStairs {
    const NAME: &'static str = "minecraft:exposed_cut_copper_stairs";
    const ID: BlockId = BlockId(1151);
    const FIRST_STATE: StateId = StateId(31245);
    const STATE_COUNT: u16 = 80;
    const DEFAULT_STATE: StateId = StateId(31256);
    const PROPERTIES: &'static [Property] = &[
        Property { name: "facing", values: &["north", "south", "west", "east"] },
        Property { name: "half", values: &["top", "bottom"] },
        Property {
            name: "shape",
            values: &[
                "straight", "inner_left", "inner_right", "outer_left", "outer_right",
            ],
        },
        Property { name: "waterlogged", values: &["true", "false"] },
    ];
    const ITEM: Option<u16> = Some(148);
    const WALL_VARIANT: Option<BlockId> = None;
    const SETTINGS: BlockSettings = BlockSettings {
        hardness: 3.0,
        resistance: 6.0,
        friction: 0.6,
        speed_factor: 1.0,
        jump_factor: 1.0,
        requires_correct_tool: true,
        ignited_by_lava: false,
    };
    const LIGHT_EMISSION: PerState<u8> = PerState::Uniform(0);
    const LIGHT_DAMPENING: PerState<u8> = PerState::Varies(&[
        1, 0, 1, 0, 1, 0, 1, 0, 1, 0, 1, 0, 1, 0, 1, 0, 1, 0, 1, 0, 1, 0, 1, 0, 1, 0, 1, 0, 1, 0, 1,
        0, 1, 0, 1, 0, 1, 0, 1, 0, 1, 0, 1, 0, 1, 0, 1, 0, 1, 0, 1, 0, 1, 0, 1, 0, 1, 0, 1, 0, 1, 0,
        1, 0, 1, 0, 1, 0, 1, 0, 1, 0, 1, 0, 1, 0, 1, 0, 1, 0,
    ]);
    const FLAGS: PerState<StateFlags> = PerState::Varies(&[
        StateFlags(654), StateFlags(558), StateFlags(654), StateFlags(558), StateFlags(654),
        StateFlags(558), StateFlags(654), StateFlags(558), StateFlags(654), StateFlags(558),
        StateFlags(654), StateFlags(558), StateFlags(654), StateFlags(558), StateFlags(654),
        StateFlags(558), StateFlags(654), StateFlags(558), StateFlags(654), StateFlags(558),
        StateFlags(654), StateFlags(558), StateFlags(654), StateFlags(558), StateFlags(654),
        StateFlags(558), StateFlags(654), StateFlags(558), StateFlags(654), StateFlags(558),
        StateFlags(654), StateFlags(558), StateFlags(654), StateFlags(558), StateFlags(654),
        StateFlags(558), StateFlags(654), StateFlags(558), StateFlags(654), StateFlags(558),
        StateFlags(654), StateFlags(558), StateFlags(654), StateFlags(558), StateFlags(654),
        StateFlags(558), StateFlags(654), StateFlags(558), StateFlags(654), StateFlags(558),
        StateFlags(654), StateFlags(558), StateFlags(654), StateFlags(558), StateFlags(654),
        StateFlags(558), StateFlags(654), StateFlags(558), StateFlags(654), StateFlags(558),
        StateFlags(654), StateFlags(558), StateFlags(654), StateFlags(558), StateFlags(654),
        StateFlags(558), StateFlags(654), StateFlags(558), StateFlags(654), StateFlags(558),
        StateFlags(654), StateFlags(558), StateFlags(654), StateFlags(558), StateFlags(654),
        StateFlags(558), StateFlags(654), StateFlags(558), StateFlags(654), StateFlags(558),
    ]);
    type Behavior = crate::family::Stairs;
}
