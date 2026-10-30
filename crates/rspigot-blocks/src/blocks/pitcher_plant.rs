use crate::{
    Block, BlockId, BlockSettings, FaceSupport, PerState, Property, ShapeId, StateFlags, StateId,
};

pub struct PitcherPlant;

impl Block for PitcherPlant {
    const NAME: &'static str = "minecraft:pitcher_plant";
    const ID: BlockId = BlockId(722);
    const FIRST_STATE: StateId = StateId(16790);
    const STATE_COUNT: u16 = 2;
    const DEFAULT_STATE: StateId = StateId(16791);
    const PROPERTIES: &'static [Property] = &[
        Property { name: "half", values: &["upper", "lower"] },
    ];
    const ITEM: Option<u16> = Some(315);
    const WALL_VARIANT: Option<BlockId> = None;
    const SETTINGS: BlockSettings = BlockSettings {
        hardness: 0.0,
        resistance: 0.0,
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
    const FLAGS: PerState<StateFlags> = PerState::Uniform(StateFlags(16416));
    const COLLISION: PerState<ShapeId> = PerState::Uniform(ShapeId(0));
    const FACE_SUPPORT: PerState<FaceSupport> = PerState::Uniform(FaceSupport(0, 0, 0));
    type Behavior = crate::family::Plain;
}
