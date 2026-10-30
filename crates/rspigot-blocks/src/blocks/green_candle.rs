use crate::{
    Block, BlockId, BlockSettings, FaceSupport, PerState, Property, ShapeId, StateFlags, StateId,
};

pub struct GreenCandle;

impl Block for GreenCandle {
    const NAME: &'static str = "minecraft:green_candle";
    const ID: BlockId = BlockId(1048);
    const FIRST_STATE: StateId = StateId(26677);
    const STATE_COUNT: u16 = 16;
    const DEFAULT_STATE: StateId = StateId(26680);
    const PROPERTIES: &'static [Property] = &[
        Property { name: "candles", values: &["1", "2", "3", "4"] },
        Property { name: "lit", values: &["true", "false"] },
        Property { name: "waterlogged", values: &["true", "false"] },
    ];
    const ITEM: Option<u16> = Some(1564);
    const WALL_VARIANT: Option<BlockId> = None;
    const SETTINGS: BlockSettings = BlockSettings {
        hardness: 0.1,
        resistance: 0.1,
        friction: 0.6,
        speed_factor: 1.0,
        jump_factor: 1.0,
        bounce_restitution: 0.0,
        fall_distance_reduction: 0.0,
        requires_correct_tool: false,
        ignited_by_lava: false,
    };
    const LIGHT_EMISSION: PerState<u8> = PerState::Varies(&[
        3, 3, 0, 0, 6, 6, 0, 0, 9, 9, 0, 0, 12, 12, 0, 0,
    ]);
    const LIGHT_DAMPENING: PerState<u8> = PerState::Varies(&[
        1, 0, 1, 0, 1, 0, 1, 0, 1, 0, 1, 0, 1, 0, 1, 0,
    ]);
    const FLAGS: PerState<StateFlags> = PerState::Varies(&[
        StateFlags(132), StateFlags(36), StateFlags(132), StateFlags(36), StateFlags(132),
        StateFlags(36), StateFlags(132), StateFlags(36), StateFlags(132), StateFlags(36),
        StateFlags(132), StateFlags(36), StateFlags(132), StateFlags(36), StateFlags(132),
        StateFlags(36),
    ]);
    const COLLISION: PerState<ShapeId> = PerState::Varies(crate::shapes::COLLISION_CANDLE);
    const FACE_SUPPORT: PerState<FaceSupport> = PerState::Uniform(FaceSupport(0, 1, 0));
    type Behavior = crate::family::Plain;
}
