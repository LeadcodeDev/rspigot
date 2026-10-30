use crate::{
    Block, BlockId, BlockSettings, FaceSupport, PerState, Property, ShapeId, StateFlags, StateId,
};

pub struct PlayerHead;

impl Block for PlayerHead {
    const NAME: &'static str = "minecraft:player_head";
    const ID: BlockId = BlockId(512);
    const FIRST_STATE: StateId = StateId(12802);
    const STATE_COUNT: u16 = 32;
    const DEFAULT_STATE: StateId = StateId(12818);
    const PROPERTIES: &'static [Property] = &[
        Property { name: "powered", values: &["true", "false"] },
        Property {
            name: "rotation",
            values: &[
                "0", "1", "2", "3", "4", "5", "6", "7", "8", "9", "10", "11", "12", "13", "14",
                "15",
            ],
        },
    ];
    const ITEM: Option<u16> = Some(1386);
    const WALL_VARIANT: Option<BlockId> = Some(BlockId(513));
    const SETTINGS: BlockSettings = BlockSettings {
        hardness: 1.0,
        resistance: 1.0,
        friction: 0.6,
        speed_factor: 1.0,
        jump_factor: 1.0,
        bounce_restitution: 0.0,
        fall_distance_reduction: 0.0,
        requires_correct_tool: false,
        ignited_by_lava: false,
    };
    const LIGHT_EMISSION: PerState<u8> = PerState::Uniform(0);
    const LIGHT_DAMPENING: PerState<u8> = PerState::Uniform(0);
    const FLAGS: PerState<StateFlags> = PerState::Uniform(StateFlags(36));
    const COLLISION: PerState<ShapeId> = PerState::Uniform(ShapeId(178));
    const FACE_SUPPORT: PerState<FaceSupport> = PerState::Uniform(FaceSupport(0, 1, 0));
    type Behavior = crate::family::StandingWithWall;
}
