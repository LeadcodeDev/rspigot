use crate::{
    Block, BlockId, BlockSettings, FaceSupport, PerState, Property, ShapeId, StateFlags, StateId,
};

pub struct Bamboo;

impl Block for Bamboo {
    const NAME: &'static str = "minecraft:bamboo";
    const ID: BlockId = BlockId(882);
    const FIRST_STATE: StateId = StateId(18636);
    const STATE_COUNT: u16 = 12;
    const DEFAULT_STATE: StateId = StateId(18636);
    const PROPERTIES: &'static [Property] = &[
        Property { name: "age", values: &["0", "1"] },
        Property { name: "leaves", values: &["none", "small", "large"] },
        Property { name: "stage", values: &["0", "1"] },
    ];
    const ITEM: Option<u16> = Some(340);
    const WALL_VARIANT: Option<BlockId> = None;
    const SETTINGS: BlockSettings = BlockSettings {
        hardness: 1.0,
        resistance: 1.0,
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
    const FLAGS: PerState<StateFlags> = PerState::Varies(&[
        StateFlags(2598), StateFlags(2086), StateFlags(2598), StateFlags(2086), StateFlags(2598),
        StateFlags(2086), StateFlags(2598), StateFlags(2086), StateFlags(2598), StateFlags(2086),
        StateFlags(2598), StateFlags(2086),
    ]);
    const COLLISION: PerState<ShapeId> = PerState::Uniform(ShapeId(274));
    const FACE_SUPPORT: PerState<FaceSupport> = PerState::Uniform(FaceSupport(0, 0, 0));
    type Behavior = crate::family::Plain;
}
