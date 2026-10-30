use crate::{Block, BlockId, BlockSettings, PerState, Property, StateFlags, StateId};

pub struct WaxedOxidizedCopperGolemStatue;

impl Block for WaxedOxidizedCopperGolemStatue {
    const NAME: &'static str = "minecraft:waxed_oxidized_copper_golem_statue";
    const ID: BlockId = BlockId(1213);
    const FIRST_STATE: StateId = StateId(33341);
    const STATE_COUNT: u16 = 32;
    const DEFAULT_STATE: StateId = StateId(33342);
    const PROPERTIES: &'static [Property] = &[
        Property { name: "copper_golem_pose", values: &["standing", "sitting", "running", "star"] },
        Property { name: "facing", values: &["north", "south", "west", "east"] },
        Property { name: "waterlogged", values: &["true", "false"] },
    ];
    const ITEM: Option<u16> = Some(1652);
    const WALL_VARIANT: Option<BlockId> = None;
    const SETTINGS: BlockSettings = BlockSettings {
        hardness: 3.0,
        resistance: 6.0,
        friction: 0.6,
        speed_factor: 1.0,
        jump_factor: 1.0,
        requires_correct_tool: false,
        ignited_by_lava: false,
    };
    const LIGHT_EMISSION: PerState<u8> = PerState::Uniform(0);
    const LIGHT_DAMPENING: PerState<u8> = PerState::Varies(&[
        1, 0, 1, 0, 1, 0, 1, 0, 1, 0, 1, 0, 1, 0, 1, 0, 1, 0, 1, 0, 1, 0, 1, 0, 1, 0, 1, 0, 1, 0, 1,
        0,
    ]);
    const FLAGS: PerState<StateFlags> = PerState::Varies(&[
        StateFlags(132), StateFlags(36), StateFlags(132), StateFlags(36), StateFlags(132),
        StateFlags(36), StateFlags(132), StateFlags(36), StateFlags(132), StateFlags(36),
        StateFlags(132), StateFlags(36), StateFlags(132), StateFlags(36), StateFlags(132),
        StateFlags(36), StateFlags(132), StateFlags(36), StateFlags(132), StateFlags(36),
        StateFlags(132), StateFlags(36), StateFlags(132), StateFlags(36), StateFlags(132),
        StateFlags(36), StateFlags(132), StateFlags(36), StateFlags(132), StateFlags(36),
        StateFlags(132), StateFlags(36),
    ]);
    type Behavior = crate::family::HorizontalAway;
}
