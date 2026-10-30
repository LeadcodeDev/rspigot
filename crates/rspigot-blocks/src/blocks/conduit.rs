use crate::{
    Block, BlockId, BlockSettings, FaceSupport, PerState, Property, ShapeId, StateFlags, StateId,
};

pub struct Conduit;

impl Block for Conduit {
    const NAME: &'static str = "minecraft:conduit";
    const ID: BlockId = BlockId(880);
    const FIRST_STATE: StateId = StateId(18633);
    const STATE_COUNT: u16 = 2;
    const DEFAULT_STATE: StateId = StateId(18633);
    const PROPERTIES: &'static [Property] = &[
        Property { name: "waterlogged", values: &["true", "false"] },
    ];
    const ITEM: Option<u16> = Some(787);
    const WALL_VARIANT: Option<BlockId> = None;
    const SETTINGS: BlockSettings = BlockSettings {
        hardness: 3.0,
        resistance: 3.0,
        friction: 0.6,
        speed_factor: 1.0,
        jump_factor: 1.0,
        bounce_restitution: 0.0,
        fall_distance_reduction: 0.0,
        requires_correct_tool: false,
        ignited_by_lava: false,
    };
    const LIGHT_EMISSION: PerState<u8> = PerState::Uniform(15);
    const LIGHT_DAMPENING: PerState<u8> = PerState::Varies(&[
        1, 0,
    ]);
    const FLAGS: PerState<StateFlags> = PerState::Varies(&[
        StateFlags(2182), StateFlags(2086),
    ]);
    const COLLISION: PerState<ShapeId> = PerState::Uniform(ShapeId(273));
    const FACE_SUPPORT: PerState<FaceSupport> = PerState::Uniform(FaceSupport(0, 0, 0));
    type Behavior = crate::family::Plain;
}
