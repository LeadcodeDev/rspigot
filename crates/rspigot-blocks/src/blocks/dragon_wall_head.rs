use crate::{
    Block, BlockId, BlockSettings, FaceSupport, PerState, Property, ShapeId, StateFlags, StateId,
};

pub struct DragonWallHead;

impl Block for DragonWallHead {
    const NAME: &'static str = "minecraft:dragon_wall_head";
    const ID: BlockId = BlockId(517);
    const FIRST_STATE: StateId = StateId(12914);
    const STATE_COUNT: u16 = 8;
    const DEFAULT_STATE: StateId = StateId(12915);
    const PROPERTIES: &'static [Property] = &[
        Property { name: "facing", values: &["north", "south", "west", "east"] },
        Property { name: "powered", values: &["true", "false"] },
    ];
    const ITEM: Option<u16> = Some(1389);
    const WALL_VARIANT: Option<BlockId> = None;
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
    const FLAGS: PerState<StateFlags> = PerState::Uniform(StateFlags(44));
    const COLLISION: PerState<ShapeId> =
        PerState::Varies(crate::shapes::COLLISION_SKELETON_WALL_SKULL);
    const FACE_SUPPORT: PerState<FaceSupport> = PerState::Uniform(FaceSupport(0, 0, 0));
    type Behavior = crate::family::Plain;
}
