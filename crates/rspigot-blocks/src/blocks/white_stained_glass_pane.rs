use crate::{
    Block, BlockId, BlockSettings, FaceSupport, PerState, Property, ShapeId, StateFlags, StateId,
};

pub struct WhiteStainedGlassPane;

impl Block for WhiteStainedGlassPane {
    const NAME: &'static str = "minecraft:white_stained_glass_pane";
    const ID: BlockId = BlockId(553);
    const FIRST_STATE: StateId = StateId(13227);
    const STATE_COUNT: u16 = 32;
    const DEFAULT_STATE: StateId = StateId(13258);
    const PROPERTIES: &'static [Property] = &[
        Property { name: "east", values: &["true", "false"] },
        Property { name: "north", values: &["true", "false"] },
        Property { name: "south", values: &["true", "false"] },
        Property { name: "waterlogged", values: &["true", "false"] },
        Property { name: "west", values: &["true", "false"] },
    ];
    const ITEM: Option<u16> = Some(621);
    const WALL_VARIANT: Option<BlockId> = None;
    const SETTINGS: BlockSettings = BlockSettings {
        hardness: 0.3,
        resistance: 0.3,
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
        1, 1, 0, 0, 1, 1, 0, 0, 1, 1, 0, 0, 1, 1, 0, 0, 1, 1, 0, 0, 1, 1, 0, 0, 1, 1, 0, 0, 1, 1, 0,
        0,
    ]);
    const FLAGS: PerState<StateFlags> = PerState::Varies(&[
        StateFlags(2182), StateFlags(2182), StateFlags(2086), StateFlags(2086), StateFlags(2182),
        StateFlags(2182), StateFlags(2086), StateFlags(2086), StateFlags(2182), StateFlags(2182),
        StateFlags(2086), StateFlags(2086), StateFlags(2182), StateFlags(2182), StateFlags(2086),
        StateFlags(2086), StateFlags(2182), StateFlags(2182), StateFlags(2086), StateFlags(2086),
        StateFlags(2182), StateFlags(2182), StateFlags(2086), StateFlags(2086), StateFlags(2182),
        StateFlags(2182), StateFlags(2086), StateFlags(2086), StateFlags(2182), StateFlags(2182),
        StateFlags(2086), StateFlags(2086),
    ]);
    const COLLISION: PerState<ShapeId> = PerState::Varies(crate::shapes::COLLISION_IRON_BARS);
    const FACE_SUPPORT: PerState<FaceSupport> = PerState::Varies(crate::shapes::SUPPORT_OAK_FENCE);
    type Behavior = crate::family::Plain;
}
