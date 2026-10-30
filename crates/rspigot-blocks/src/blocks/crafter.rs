use crate::{
    Block, BlockId, BlockSettings, FaceSupport, PerState, Property, ShapeId, StateFlags, StateId,
};

pub struct Crafter;

impl Block for Crafter {
    const NAME: &'static str = "minecraft:crafter";
    const ID: BlockId = BlockId(1274);
    const FIRST_STATE: StateId = StateId(35459);
    const STATE_COUNT: u16 = 48;
    const DEFAULT_STATE: StateId = StateId(35504);
    const PROPERTIES: &'static [Property] = &[
        Property { name: "crafting", values: &["true", "false"] },
        Property {
            name: "orientation",
            values: &[
                "down_east", "down_north", "down_south", "down_west", "up_east", "up_north",
                "up_south", "up_west", "west_up", "east_up", "north_up", "south_up",
            ],
        },
        Property { name: "triggered", values: &["true", "false"] },
    ];
    const ITEM: Option<u16> = Some(1237);
    const WALL_VARIANT: Option<BlockId> = None;
    const SETTINGS: BlockSettings = BlockSettings {
        hardness: 1.5,
        resistance: 3.5,
        friction: 0.6,
        speed_factor: 1.0,
        jump_factor: 1.0,
        bounce_restitution: 0.0,
        fall_distance_reduction: 0.0,
        requires_correct_tool: false,
        ignited_by_lava: false,
    };
    const LIGHT_EMISSION: PerState<u8> = PerState::Uniform(0);
    const LIGHT_DAMPENING: PerState<u8> = PerState::Uniform(15);
    const FLAGS: PerState<StateFlags> = PerState::Uniform(StateFlags(7198));
    const COLLISION: PerState<ShapeId> = PerState::Uniform(ShapeId(1));
    const FACE_SUPPORT: PerState<FaceSupport> = PerState::Uniform(FaceSupport(63, 63, 63));
    type Behavior = crate::family::Plain;
}
