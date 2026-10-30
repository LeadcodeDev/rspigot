use crate::{Block, BlockId, BlockSettings, PerState, Property, StateFlags, StateId};

pub struct SmallAmethystBud;

impl Block for SmallAmethystBud {
    const NAME: &'static str = "minecraft:small_amethyst_bud";
    const ID: BlockId = BlockId(1073);
    const FIRST_STATE: StateId = StateId(26797);
    const STATE_COUNT: u16 = 12;
    const DEFAULT_STATE: StateId = StateId(26806);
    const PROPERTIES: &'static [Property] = &[
        Property { name: "facing", values: &["north", "east", "south", "west", "up", "down"] },
        Property { name: "waterlogged", values: &["true", "false"] },
    ];
    const ITEM: Option<u16> = Some(1567);
    const WALL_VARIANT: Option<BlockId> = None;
    const SETTINGS: BlockSettings = BlockSettings {
        hardness: 1.5,
        resistance: 1.5,
        friction: 0.6,
        speed_factor: 1.0,
        jump_factor: 1.0,
        requires_correct_tool: false,
        ignited_by_lava: false,
    };
    const LIGHT_EMISSION: PerState<u8> = PerState::Uniform(1);
    const LIGHT_DAMPENING: PerState<u8> = PerState::Varies(&[
        1, 0, 1, 0, 1, 0, 1, 0, 1, 0, 1, 0,
    ]);
    const FLAGS: PerState<StateFlags> = PerState::Varies(&[
        StateFlags(134), StateFlags(38), StateFlags(134), StateFlags(38), StateFlags(134),
        StateFlags(38), StateFlags(134), StateFlags(38), StateFlags(134), StateFlags(38),
        StateFlags(134), StateFlags(38),
    ]);
    type Behavior = crate::family::FaceAttached;
}
