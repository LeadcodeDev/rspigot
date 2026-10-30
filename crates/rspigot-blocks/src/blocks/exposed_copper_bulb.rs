use crate::{
    Block, BlockId, BlockSettings, FaceSupport, PerState, Property, ShapeId, StateFlags, StateId,
};

pub struct ExposedCopperBulb;

impl Block for ExposedCopperBulb {
    const NAME: &'static str = "minecraft:exposed_copper_bulb";
    const ID: BlockId = BlockId(1191);
    const FIRST_STATE: StateId = StateId(32897);
    const STATE_COUNT: u16 = 4;
    const DEFAULT_STATE: StateId = StateId(32900);
    const PROPERTIES: &'static [Property] = &[
        Property { name: "lit", values: &["true", "false"] },
        Property { name: "powered", values: &["true", "false"] },
    ];
    const ITEM: Option<u16> = Some(1630);
    const WALL_VARIANT: Option<BlockId> = None;
    const SETTINGS: BlockSettings = BlockSettings {
        hardness: 3.0,
        resistance: 6.0,
        friction: 0.6,
        speed_factor: 1.0,
        jump_factor: 1.0,
        bounce_restitution: 0.0,
        fall_distance_reduction: 0.0,
        requires_correct_tool: true,
        ignited_by_lava: false,
    };
    const LIGHT_EMISSION: PerState<u8> = PerState::Varies(&[
        12, 12, 0, 0,
    ]);
    const LIGHT_DAMPENING: PerState<u8> = PerState::Uniform(15);
    const FLAGS: PerState<StateFlags> = PerState::Uniform(StateFlags(3614));
    const COLLISION: PerState<ShapeId> = PerState::Uniform(ShapeId(1));
    const FACE_SUPPORT: PerState<FaceSupport> = PerState::Uniform(FaceSupport(63, 63, 63));
    type Behavior = crate::family::Plain;
}
