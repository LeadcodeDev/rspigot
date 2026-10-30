use crate::{
    Block, BlockId, BlockSettings, FaceSupport, PerState, Property, ShapeId, StateFlags, StateId,
};

pub struct Lectern;

impl Block for Lectern {
    const NAME: &'static str = "minecraft:lectern";
    const ID: BlockId = BlockId(935);
    const FIRST_STATE: StateId = StateId(24141);
    const STATE_COUNT: u16 = 16;
    const DEFAULT_STATE: StateId = StateId(24144);
    const PROPERTIES: &'static [Property] = &[
        Property { name: "facing", values: &["north", "south", "west", "east"] },
        Property { name: "has_book", values: &["true", "false"] },
        Property { name: "powered", values: &["true", "false"] },
    ];
    const ITEM: Option<u16> = Some(837);
    const WALL_VARIANT: Option<BlockId> = None;
    const SETTINGS: BlockSettings = BlockSettings {
        hardness: 2.5,
        resistance: 2.5,
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
    const COLLISION: PerState<ShapeId> = PerState::Uniform(ShapeId(284));
    const FACE_SUPPORT: PerState<FaceSupport> = PerState::Uniform(FaceSupport(1, 1, 1));
    type Behavior = crate::family::HorizontalAway;
}
