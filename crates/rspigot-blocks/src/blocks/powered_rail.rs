use crate::{
    Block, BlockId, BlockSettings, FaceSupport, PerState, Property, ShapeId, StateFlags, StateId,
};

pub struct PoweredRail;

impl Block for PoweredRail {
    const NAME: &'static str = "minecraft:powered_rail";
    const ID: BlockId = BlockId(136);
    const FIRST_STATE: StateId = StateId(2302);
    const STATE_COUNT: u16 = 24;
    const DEFAULT_STATE: StateId = StateId(2315);
    const PROPERTIES: &'static [Property] = &[
        Property { name: "powered", values: &["true", "false"] },
        Property {
            name: "shape",
            values: &[
                "north_south", "east_west", "ascending_east", "ascending_west", "ascending_north",
                "ascending_south",
            ],
        },
        Property { name: "waterlogged", values: &["true", "false"] },
    ];
    const ITEM: Option<u16> = Some(945);
    const WALL_VARIANT: Option<BlockId> = None;
    const SETTINGS: BlockSettings = BlockSettings {
        hardness: 0.7,
        resistance: 0.7,
        friction: 0.6,
        speed_factor: 1.0,
        jump_factor: 1.0,
        bounce_restitution: 0.0,
        fall_distance_reduction: 0.0,
        requires_correct_tool: false,
        ignited_by_lava: false,
    };
    const LIGHT_EMISSION: PerState<u8> = PerState::Uniform(0);
    const LIGHT_DAMPENING: PerState<u8> = PerState::Varies(&[
        1, 0, 1, 0, 1, 0, 1, 0, 1, 0, 1, 0, 1, 0, 1, 0, 1, 0, 1, 0, 1, 0, 1, 0,
    ]);
    const FLAGS: PerState<StateFlags> = PerState::Varies(&[
        StateFlags(128), StateFlags(32), StateFlags(128), StateFlags(32), StateFlags(128),
        StateFlags(32), StateFlags(128), StateFlags(32), StateFlags(128), StateFlags(32),
        StateFlags(128), StateFlags(32), StateFlags(128), StateFlags(32), StateFlags(128),
        StateFlags(32), StateFlags(128), StateFlags(32), StateFlags(128), StateFlags(32),
        StateFlags(128), StateFlags(32), StateFlags(128), StateFlags(32),
    ]);
    const COLLISION: PerState<ShapeId> = PerState::Uniform(ShapeId(0));
    const FACE_SUPPORT: PerState<FaceSupport> = PerState::Uniform(FaceSupport(0, 0, 0));
    type Behavior = crate::family::Plain;
}
