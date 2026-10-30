use crate::{
    Block, BlockId, BlockSettings, FaceSupport, PerState, Property, ShapeId, StateFlags, StateId,
};

pub struct DaylightDetector;

impl Block for DaylightDetector {
    const NAME: &'static str = "minecraft:daylight_detector";
    const ID: BlockId = BlockId(527);
    const FIRST_STATE: StateId = StateId(13046);
    const STATE_COUNT: u16 = 32;
    const DEFAULT_STATE: StateId = StateId(13062);
    const PROPERTIES: &'static [Property] = &[
        Property { name: "inverted", values: &["true", "false"] },
        Property {
            name: "power",
            values: &[
                "0", "1", "2", "3", "4", "5", "6", "7", "8", "9", "10", "11", "12", "13", "14",
                "15",
            ],
        },
    ];
    const ITEM: Option<u16> = Some(848);
    const WALL_VARIANT: Option<BlockId> = None;
    const SETTINGS: BlockSettings = BlockSettings {
        hardness: 0.2,
        resistance: 0.2,
        friction: 0.6,
        speed_factor: 1.0,
        jump_factor: 1.0,
        bounce_restitution: 0.0,
        fall_distance_reduction: 0.0,
        requires_correct_tool: false,
        ignited_by_lava: true,
    };
    const LIGHT_EMISSION: PerState<u8> = PerState::Uniform(0);
    const LIGHT_DAMPENING: PerState<u8> = PerState::Uniform(0);
    const FLAGS: PerState<StateFlags> = PerState::Uniform(StateFlags(2094));
    const COLLISION: PerState<ShapeId> = PerState::Uniform(ShapeId(72));
    const FACE_SUPPORT: PerState<FaceSupport> = PerState::Uniform(FaceSupport(1, 1, 1));
    type Behavior = crate::family::Plain;
}
