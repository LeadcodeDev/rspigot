use crate::{
    Block, BlockId, BlockSettings, FaceSupport, PerState, Property, ShapeId, StateFlags, StateId,
};

pub struct PitcherCrop;

impl Block for PitcherCrop {
    const NAME: &'static str = "minecraft:pitcher_crop";
    const ID: BlockId = BlockId(721);
    const FIRST_STATE: StateId = StateId(16780);
    const STATE_COUNT: u16 = 10;
    const DEFAULT_STATE: StateId = StateId(16781);
    const PROPERTIES: &'static [Property] = &[
        Property { name: "age", values: &["0", "1", "2", "3", "4"] },
        Property { name: "half", values: &["upper", "lower"] },
    ];
    const ITEM: Option<u16> = Some(1437);
    const WALL_VARIANT: Option<BlockId> = None;
    const SETTINGS: BlockSettings = BlockSettings {
        hardness: 0.0,
        resistance: 0.0,
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
    const FLAGS: PerState<StateFlags> = PerState::Varies(&[
        StateFlags(32), StateFlags(548), StateFlags(32), StateFlags(548), StateFlags(32),
        StateFlags(548), StateFlags(32), StateFlags(548), StateFlags(32), StateFlags(36),
    ]);
    const COLLISION: PerState<ShapeId> = PerState::Varies(crate::shapes::COLLISION_PITCHER_CROP);
    const FACE_SUPPORT: PerState<FaceSupport> =
        PerState::Varies(crate::shapes::SUPPORT_PITCHER_CROP);
    type Behavior = crate::family::Plain;
}
