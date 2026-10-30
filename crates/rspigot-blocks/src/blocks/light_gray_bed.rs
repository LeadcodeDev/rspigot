use crate::{
    Block, BlockId, BlockSettings, FaceSupport, PerState, Property, ShapeId, StateFlags, StateId,
};

pub struct LightGrayBed;

impl Block for LightGrayBed {
    const NAME: &'static str = "minecraft:light_gray_bed";
    const ID: BlockId = BlockId(127);
    const FIRST_STATE: StateId = StateId(2158);
    const STATE_COUNT: u16 = 16;
    const DEFAULT_STATE: StateId = StateId(2161);
    const PROPERTIES: &'static [Property] = &[
        Property { name: "facing", values: &["north", "south", "west", "east"] },
        Property { name: "occupied", values: &["true", "false"] },
        Property { name: "part", values: &["head", "foot"] },
    ];
    const ITEM: Option<u16> = Some(1227);
    const WALL_VARIANT: Option<BlockId> = None;
    const SETTINGS: BlockSettings = BlockSettings {
        hardness: 0.2,
        resistance: 0.2,
        friction: 0.6,
        speed_factor: 1.0,
        jump_factor: 1.0,
        bounce_restitution: 0.75,
        fall_distance_reduction: 0.5,
        requires_correct_tool: false,
        ignited_by_lava: true,
    };
    const LIGHT_EMISSION: PerState<u8> = PerState::Uniform(0);
    const LIGHT_DAMPENING: PerState<u8> = PerState::Uniform(0);
    const FLAGS: PerState<StateFlags> = PerState::Uniform(StateFlags(2086));
    const COLLISION: PerState<ShapeId> = PerState::Varies(crate::shapes::COLLISION_WHITE_BED);
    const FACE_SUPPORT: PerState<FaceSupport> = PerState::Uniform(FaceSupport(0, 0, 0));
    type Behavior = crate::family::HorizontalToward;
}
