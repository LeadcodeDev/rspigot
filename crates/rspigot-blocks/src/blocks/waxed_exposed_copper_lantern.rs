use crate::{
    Block, BlockId, BlockSettings, FaceSupport, PerState, Property, ShapeId, StateFlags, StateId,
};

pub struct WaxedExposedCopperLantern;

impl Block for WaxedExposedCopperLantern {
    const NAME: &'static str = "minecraft:waxed_exposed_copper_lantern";
    const ID: BlockId = BlockId(946);
    const FIRST_STATE: StateId = StateId(24222);
    const STATE_COUNT: u16 = 4;
    const DEFAULT_STATE: StateId = StateId(24225);
    const PROPERTIES: &'static [Property] = &[
        Property { name: "hanging", values: &["true", "false"] },
        Property { name: "waterlogged", values: &["true", "false"] },
    ];
    const ITEM: Option<u16> = Some(1522);
    const WALL_VARIANT: Option<BlockId> = None;
    const SETTINGS: BlockSettings = BlockSettings {
        hardness: 3.5,
        resistance: 3.5,
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
        1, 0, 1, 0,
    ]);
    const FLAGS: PerState<StateFlags> = PerState::Varies(&[
        StateFlags(2182), StateFlags(2086), StateFlags(2182), StateFlags(2086),
    ]);
    const COLLISION: PerState<ShapeId> = PerState::Varies(crate::shapes::COLLISION_LANTERN);
    const FACE_SUPPORT: PerState<FaceSupport> = PerState::Varies(crate::shapes::SUPPORT_LANTERN);
    type Behavior = crate::family::Plain;
}
