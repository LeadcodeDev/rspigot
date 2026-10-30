use crate::{
    Block, BlockId, BlockSettings, FaceSupport, PerState, Property, ShapeId, StateFlags, StateId,
};

pub struct RespawnAnchor;

impl Block for RespawnAnchor {
    const NAME: &'static str = "minecraft:respawn_anchor";
    const ID: BlockId = BlockId(1008);
    const FIRST_STATE: StateId = StateId(25178);
    const STATE_COUNT: u16 = 5;
    const DEFAULT_STATE: StateId = StateId(25178);
    const PROPERTIES: &'static [Property] = &[
        Property { name: "charges", values: &["0", "1", "2", "3", "4"] },
    ];
    const ITEM: Option<u16> = Some(1549);
    const WALL_VARIANT: Option<BlockId> = None;
    const SETTINGS: BlockSettings = BlockSettings {
        hardness: 50.0,
        resistance: 1200.0,
        friction: 0.6,
        speed_factor: 1.0,
        jump_factor: 1.0,
        bounce_restitution: 0.0,
        fall_distance_reduction: 0.0,
        requires_correct_tool: true,
        ignited_by_lava: false,
    };
    const LIGHT_EMISSION: PerState<u8> = PerState::Varies(&[
        0, 3, 7, 11, 15,
    ]);
    const LIGHT_DAMPENING: PerState<u8> = PerState::Uniform(15);
    const FLAGS: PerState<StateFlags> = PerState::Uniform(StateFlags(7198));
    const COLLISION: PerState<ShapeId> = PerState::Uniform(ShapeId(1));
    const FACE_SUPPORT: PerState<FaceSupport> = PerState::Uniform(FaceSupport(63, 63, 63));
    type Behavior = crate::family::Plain;
}
