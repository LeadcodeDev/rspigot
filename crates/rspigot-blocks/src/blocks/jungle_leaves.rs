use crate::{Block, BlockId, BlockSettings, PerState, Property, StateFlags, StateId};

pub struct JungleLeaves;

impl Block for JungleLeaves {
    const NAME: &'static str = "minecraft:jungle_leaves";
    const ID: BlockId = BlockId(97);
    const FIRST_STATE: StateId = StateId(351);
    const STATE_COUNT: u16 = 28;
    const DEFAULT_STATE: StateId = StateId(378);
    const PROPERTIES: &'static [Property] = &[
        Property { name: "distance", values: &["1", "2", "3", "4", "5", "6", "7"] },
        Property { name: "persistent", values: &["true", "false"] },
        Property { name: "waterlogged", values: &["true", "false"] },
    ];
    const ITEM: Option<u16> = Some(218);
    const WALL_VARIANT: Option<BlockId> = None;
    const SETTINGS: BlockSettings = BlockSettings {
        hardness: 0.2,
        resistance: 0.2,
        friction: 0.6,
        speed_factor: 1.0,
        jump_factor: 1.0,
        requires_correct_tool: false,
        ignited_by_lava: true,
    };
    const LIGHT_EMISSION: PerState<u8> = PerState::Uniform(0);
    const LIGHT_DAMPENING: PerState<u8> = PerState::Uniform(1);
    const FLAGS: PerState<StateFlags> = PerState::Varies(&[
        StateFlags(1158), StateFlags(1030), StateFlags(1158), StateFlags(1030), StateFlags(1158),
        StateFlags(1030), StateFlags(1158), StateFlags(1030), StateFlags(1158), StateFlags(1030),
        StateFlags(1158), StateFlags(1030), StateFlags(1158), StateFlags(1030), StateFlags(1158),
        StateFlags(1030), StateFlags(1158), StateFlags(1030), StateFlags(1158), StateFlags(1030),
        StateFlags(1158), StateFlags(1030), StateFlags(1158), StateFlags(1030), StateFlags(1158),
        StateFlags(1030), StateFlags(1670), StateFlags(1542),
    ]);
    type Behavior = crate::family::Plain;
}
