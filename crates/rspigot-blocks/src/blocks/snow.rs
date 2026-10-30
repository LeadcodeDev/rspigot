use crate::{
    Block, BlockId, BlockSettings, FaceSupport, PerState, Property, ShapeId, StateFlags, StateId,
};

pub struct Snow;

impl Block for Snow {
    const NAME: &'static str = "minecraft:snow";
    const ID: BlockId = BlockId(325);
    const FIRST_STATE: StateId = StateId(8589);
    const STATE_COUNT: u16 = 8;
    const DEFAULT_STATE: StateId = StateId(8589);
    const PROPERTIES: &'static [Property] = &[
        Property { name: "layers", values: &["1", "2", "3", "4", "5", "6", "7", "8"] },
    ];
    const ITEM: Option<u16> = Some(410);
    const WALL_VARIANT: Option<BlockId> = None;
    const SETTINGS: BlockSettings = BlockSettings {
        hardness: 0.1,
        resistance: 0.1,
        friction: 0.6,
        speed_factor: 1.0,
        jump_factor: 1.0,
        bounce_restitution: 0.0,
        fall_distance_reduction: 0.0,
        requires_correct_tool: true,
        ignited_by_lava: false,
    };
    const LIGHT_EMISSION: PerState<u8> = PerState::Uniform(0);
    const LIGHT_DAMPENING: PerState<u8> = PerState::Varies(&[
        0, 0, 0, 0, 0, 0, 0, 15,
    ]);
    const FLAGS: PerState<StateFlags> = PerState::Varies(&[
        StateFlags(17000), StateFlags(17004), StateFlags(17004), StateFlags(17004), StateFlags(620),
        StateFlags(620), StateFlags(620), StateFlags(604),
    ]);
    const COLLISION: PerState<ShapeId> = PerState::Varies(crate::shapes::COLLISION_SNOW);
    const FACE_SUPPORT: PerState<FaceSupport> = PerState::Varies(crate::shapes::SUPPORT_SNOW);
    type Behavior = crate::family::Plain;
}
