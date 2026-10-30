use crate::{
    Block, BlockId, BlockSettings, FaceSupport, PerState, Property, ShapeId, StateFlags, StateId,
};

pub struct SnifferEgg;

impl Block for SnifferEgg {
    const NAME: &'static str = "minecraft:sniffer_egg";
    const ID: BlockId = BlockId(836);
    const FIRST_STATE: StateId = StateId(18459);
    const STATE_COUNT: u16 = 3;
    const DEFAULT_STATE: StateId = StateId(18459);
    const PROPERTIES: &'static [Property] = &[
        Property { name: "hatch", values: &["0", "1", "2"] },
    ];
    const ITEM: Option<u16> = Some(754);
    const WALL_VARIANT: Option<BlockId> = None;
    const SETTINGS: BlockSettings = BlockSettings {
        hardness: 0.5,
        resistance: 0.5,
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
    const FLAGS: PerState<StateFlags> = PerState::Uniform(StateFlags(38));
    const COLLISION: PerState<ShapeId> = PerState::Uniform(ShapeId(267));
    const FACE_SUPPORT: PerState<FaceSupport> = PerState::Uniform(FaceSupport(0, 3, 0));
    type Behavior = crate::family::Plain;
}
