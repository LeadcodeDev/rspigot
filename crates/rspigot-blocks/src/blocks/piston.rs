use crate::{
    Block, BlockId, BlockSettings, FaceSupport, PerState, Property, ShapeId, StateFlags, StateId,
};

pub struct Piston;

impl Block for Piston {
    const NAME: &'static str = "minecraft:piston";
    const ID: BlockId = BlockId(149);
    const FIRST_STATE: StateId = StateId(2373);
    const STATE_COUNT: u16 = 12;
    const DEFAULT_STATE: StateId = StateId(2379);
    const PROPERTIES: &'static [Property] = &[
        Property { name: "extended", values: &["true", "false"] },
        Property { name: "facing", values: &["north", "east", "south", "west", "up", "down"] },
    ];
    const ITEM: Option<u16> = Some(829);
    const WALL_VARIANT: Option<BlockId> = None;
    const SETTINGS: BlockSettings = BlockSettings {
        hardness: 1.5,
        resistance: 1.5,
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
        0, 0, 0, 0, 0, 0, 15, 15, 15, 15, 15, 15,
    ]);
    const FLAGS: PerState<StateFlags> = PerState::Varies(&[
        StateFlags(46), StateFlags(46), StateFlags(46), StateFlags(46), StateFlags(46),
        StateFlags(46), StateFlags(1054), StateFlags(1054), StateFlags(1054), StateFlags(1054),
        StateFlags(1054), StateFlags(1054),
    ]);
    const COLLISION: PerState<ShapeId> = PerState::Varies(crate::shapes::COLLISION_STICKY_PISTON);
    const FACE_SUPPORT: PerState<FaceSupport> =
        PerState::Varies(crate::shapes::SUPPORT_STICKY_PISTON);
    type Behavior = crate::family::Directional<true>;
}
