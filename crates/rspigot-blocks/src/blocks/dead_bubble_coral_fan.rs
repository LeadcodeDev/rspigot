use crate::{Block, BlockId, BlockSettings, PerState, Property, StateFlags, StateId};

pub struct DeadBubbleCoralFan;

impl Block for DeadBubbleCoralFan {
    const NAME: &'static str = "minecraft:dead_bubble_coral_fan";
    const ID: BlockId = BlockId(860);
    const FIRST_STATE: StateId = StateId(18528);
    const STATE_COUNT: u16 = 2;
    const DEFAULT_STATE: StateId = StateId(18528);
    const PROPERTIES: &'static [Property] = &[
        Property { name: "waterlogged", values: &["true", "false"] },
    ];
    const ITEM: Option<u16> = Some(783);
    const WALL_VARIANT: Option<BlockId> = Some(BlockId(870));
    const SETTINGS: BlockSettings = BlockSettings {
        hardness: 0.0,
        resistance: 0.0,
        friction: 0.6,
        speed_factor: 1.0,
        jump_factor: 1.0,
        requires_correct_tool: true,
        ignited_by_lava: false,
    };
    const LIGHT_EMISSION: PerState<u8> = PerState::Uniform(0);
    const LIGHT_DAMPENING: PerState<u8> = PerState::Varies(&[
        1, 0,
    ]);
    const FLAGS: PerState<StateFlags> = PerState::Varies(&[
        StateFlags(130), StateFlags(34),
    ]);
    type Behavior = crate::family::StandingWithWall;
}
