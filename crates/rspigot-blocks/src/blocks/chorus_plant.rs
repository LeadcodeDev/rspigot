use crate::{
    Block, BlockId, BlockSettings, FaceSupport, PerState, Property, ShapeId, StateFlags, StateId,
};

pub struct ChorusPlant;

impl Block for ChorusPlant {
    const NAME: &'static str = "minecraft:chorus_plant";
    const ID: BlockId = BlockId(713);
    const FIRST_STATE: StateId = StateId(16617);
    const STATE_COUNT: u16 = 64;
    const DEFAULT_STATE: StateId = StateId(16680);
    const PROPERTIES: &'static [Property] = &[
        Property { name: "down", values: &["true", "false"] },
        Property { name: "east", values: &["true", "false"] },
        Property { name: "north", values: &["true", "false"] },
        Property { name: "south", values: &["true", "false"] },
        Property { name: "up", values: &["true", "false"] },
        Property { name: "west", values: &["true", "false"] },
    ];
    const ITEM: Option<u16> = Some(397);
    const WALL_VARIANT: Option<BlockId> = None;
    const SETTINGS: BlockSettings = BlockSettings {
        hardness: 0.4,
        resistance: 0.4,
        friction: 0.6,
        speed_factor: 1.0,
        jump_factor: 1.0,
        requires_correct_tool: false,
        ignited_by_lava: false,
    };
    const LIGHT_EMISSION: PerState<u8> = PerState::Uniform(0);
    const LIGHT_DAMPENING: PerState<u8> = PerState::Uniform(1);
    const FLAGS: PerState<StateFlags> = PerState::Uniform(StateFlags(4));
    const COLLISION: PerState<ShapeId> = PerState::Varies(crate::shapes::COLLISION_CHORUS_PLANT);
    const FACE_SUPPORT: PerState<FaceSupport> =
        PerState::Varies(crate::shapes::SUPPORT_CHORUS_PLANT);
    type Behavior = crate::family::Plain;
}
