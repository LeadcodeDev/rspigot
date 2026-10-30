use crate::{
    Block, BlockId, BlockSettings, FaceSupport, PerState, Property, ShapeId, StateFlags, StateId,
};

pub struct Comparator;

impl Block for Comparator {
    const NAME: &'static str = "minecraft:comparator";
    const ID: BlockId = BlockId(526);
    const FIRST_STATE: StateId = StateId(13030);
    const STATE_COUNT: u16 = 16;
    const DEFAULT_STATE: StateId = StateId(13031);
    const PROPERTIES: &'static [Property] = &[
        Property { name: "facing", values: &["north", "south", "west", "east"] },
        Property { name: "mode", values: &["compare", "subtract"] },
        Property { name: "powered", values: &["true", "false"] },
    ];
    const ITEM: Option<u16> = Some(828);
    const WALL_VARIANT: Option<BlockId> = None;
    const SETTINGS: BlockSettings = BlockSettings {
        hardness: 0.0,
        resistance: 0.0,
        friction: 0.6,
        speed_factor: 1.0,
        jump_factor: 1.0,
        requires_correct_tool: false,
        ignited_by_lava: false,
    };
    const LIGHT_EMISSION: PerState<u8> = PerState::Uniform(0);
    const LIGHT_DAMPENING: PerState<u8> = PerState::Uniform(0);
    const FLAGS: PerState<StateFlags> = PerState::Uniform(StateFlags(44));
    const COLLISION: PerState<ShapeId> = PerState::Uniform(ShapeId(71));
    const FACE_SUPPORT: PerState<FaceSupport> = PerState::Uniform(FaceSupport(1, 1, 1));
    type Behavior = crate::family::HorizontalAway;
}
