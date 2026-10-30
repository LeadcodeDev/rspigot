use crate::{
    Block, BlockId, BlockSettings, FaceSupport, PerState, Property, ShapeId, StateFlags, StateId,
};

pub struct PinkCandleCake;

impl Block for PinkCandleCake {
    const NAME: &'static str = "minecraft:pink_candle_cake";
    const ID: BlockId = BlockId(1058);
    const FIRST_STATE: StateId = StateId(26739);
    const STATE_COUNT: u16 = 2;
    const DEFAULT_STATE: StateId = StateId(26740);
    const PROPERTIES: &'static [Property] = &[
        Property { name: "lit", values: &["true", "false"] },
    ];
    const ITEM: Option<u16> = None;
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
    const LIGHT_EMISSION: PerState<u8> = PerState::Varies(&[
        3, 0,
    ]);
    const LIGHT_DAMPENING: PerState<u8> = PerState::Uniform(0);
    const FLAGS: PerState<StateFlags> = PerState::Uniform(StateFlags(46));
    const COLLISION: PerState<ShapeId> = PerState::Uniform(ShapeId(303));
    const FACE_SUPPORT: PerState<FaceSupport> = PerState::Uniform(FaceSupport(0, 1, 0));
    type Behavior = crate::family::Plain;
}
