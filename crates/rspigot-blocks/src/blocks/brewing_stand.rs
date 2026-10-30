use crate::{
    Block, BlockId, BlockSettings, FaceSupport, PerState, Property, ShapeId, StateFlags, StateId,
};

pub struct BrewingStand;

impl Block for BrewingStand {
    const NAME: &'static str = "minecraft:brewing_stand";
    const ID: BlockId = BlockId(436);
    const FIRST_STATE: StateId = StateId(11186);
    const STATE_COUNT: u16 = 8;
    const DEFAULT_STATE: StateId = StateId(11193);
    const PROPERTIES: &'static [Property] = &[
        Property { name: "has_bottle_0", values: &["true", "false"] },
        Property { name: "has_bottle_1", values: &["true", "false"] },
        Property { name: "has_bottle_2", values: &["true", "false"] },
    ];
    const ITEM: Option<u16> = Some(1276);
    const WALL_VARIANT: Option<BlockId> = None;
    const SETTINGS: BlockSettings = BlockSettings {
        hardness: 0.5,
        resistance: 0.5,
        friction: 0.6,
        speed_factor: 1.0,
        jump_factor: 1.0,
        requires_correct_tool: false,
        ignited_by_lava: false,
    };
    const LIGHT_EMISSION: PerState<u8> = PerState::Uniform(1);
    const LIGHT_DAMPENING: PerState<u8> = PerState::Uniform(0);
    const FLAGS: PerState<StateFlags> = PerState::Uniform(StateFlags(38));
    const COLLISION: PerState<ShapeId> = PerState::Uniform(ShapeId(152));
    const FACE_SUPPORT: PerState<FaceSupport> = PerState::Uniform(FaceSupport(0, 1, 0));
    type Behavior = crate::family::Plain;
}
