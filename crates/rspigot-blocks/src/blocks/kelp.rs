use crate::{
    Block, BlockId, BlockSettings, FaceSupport, PerState, Property, ShapeId, StateFlags, StateId,
};

pub struct Kelp;

impl Block for Kelp {
    const NAME: &'static str = "minecraft:kelp";
    const ID: BlockId = BlockId(832);
    const FIRST_STATE: StateId = StateId(18419);
    const STATE_COUNT: u16 = 26;
    const DEFAULT_STATE: StateId = StateId(18419);
    const PROPERTIES: &'static [Property] = &[
        Property {
            name: "age",
            values: &[
                "0", "1", "2", "3", "4", "5", "6", "7", "8", "9", "10", "11", "12", "13", "14",
                "15", "16", "17", "18", "19", "20", "21", "22", "23", "24", "25",
            ],
        },
    ];
    const ITEM: Option<u16> = Some(328);
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
        ignited_by_lava: false,
    };
    const LIGHT_EMISSION: PerState<u8> = PerState::Uniform(0);
    const LIGHT_DAMPENING: PerState<u8> = PerState::Uniform(1);
    const FLAGS: PerState<StateFlags> = PerState::Varies(&[
        StateFlags(17024), StateFlags(17024), StateFlags(17024), StateFlags(17024),
        StateFlags(17024), StateFlags(17024), StateFlags(17024), StateFlags(17024),
        StateFlags(17024), StateFlags(17024), StateFlags(17024), StateFlags(17024),
        StateFlags(17024), StateFlags(17024), StateFlags(17024), StateFlags(17024),
        StateFlags(17024), StateFlags(17024), StateFlags(17024), StateFlags(17024),
        StateFlags(17024), StateFlags(17024), StateFlags(17024), StateFlags(17024),
        StateFlags(17024), StateFlags(16512),
    ]);
    const COLLISION: PerState<ShapeId> = PerState::Uniform(ShapeId(0));
    const FACE_SUPPORT: PerState<FaceSupport> = PerState::Uniform(FaceSupport(0, 0, 0));
    type Behavior = crate::family::Plain;
}
