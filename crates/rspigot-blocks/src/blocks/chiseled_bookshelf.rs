use crate::{
    Block, BlockId, BlockSettings, FaceSupport, PerState, Property, ShapeId, StateFlags, StateId,
};

pub struct ChiseledBookshelf;

impl Block for ChiseledBookshelf {
    const NAME: &'static str = "minecraft:chiseled_bookshelf";
    const ID: BlockId = BlockId(222);
    const FIRST_STATE: StateId = StateId(3836);
    const STATE_COUNT: u16 = 256;
    const DEFAULT_STATE: StateId = StateId(3899);
    const PROPERTIES: &'static [Property] = &[
        Property { name: "facing", values: &["north", "south", "west", "east"] },
        Property { name: "slot_0_occupied", values: &["true", "false"] },
        Property { name: "slot_1_occupied", values: &["true", "false"] },
        Property { name: "slot_2_occupied", values: &["true", "false"] },
        Property { name: "slot_3_occupied", values: &["true", "false"] },
        Property { name: "slot_4_occupied", values: &["true", "false"] },
        Property { name: "slot_5_occupied", values: &["true", "false"] },
    ];
    const ITEM: Option<u16> = Some(391);
    const WALL_VARIANT: Option<BlockId> = None;
    const SETTINGS: BlockSettings = BlockSettings {
        hardness: 1.5,
        resistance: 1.5,
        friction: 0.6,
        speed_factor: 1.0,
        jump_factor: 1.0,
        bounce_restitution: 0.0,
        fall_distance_reduction: 0.0,
        requires_correct_tool: false,
        ignited_by_lava: true,
    };
    const LIGHT_EMISSION: PerState<u8> = PerState::Uniform(0);
    const LIGHT_DAMPENING: PerState<u8> = PerState::Uniform(15);
    const FLAGS: PerState<StateFlags> = PerState::Uniform(StateFlags(1054));
    const COLLISION: PerState<ShapeId> = PerState::Uniform(ShapeId(1));
    const FACE_SUPPORT: PerState<FaceSupport> = PerState::Uniform(FaceSupport(63, 63, 63));
    type Behavior = crate::family::HorizontalAway;
}
