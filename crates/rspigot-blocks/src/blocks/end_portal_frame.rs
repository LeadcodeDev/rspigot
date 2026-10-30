use crate::{
    Block, BlockId, BlockSettings, FaceSupport, PerState, Property, ShapeId, StateFlags, StateId,
};

pub struct EndPortalFrame;

impl Block for EndPortalFrame {
    const NAME: &'static str = "minecraft:end_portal_frame";
    const ID: BlockId = BlockId(442);
    const FIRST_STATE: StateId = StateId(11203);
    const STATE_COUNT: u16 = 8;
    const DEFAULT_STATE: StateId = StateId(11207);
    const PROPERTIES: &'static [Property] = &[
        Property { name: "eye", values: &["true", "false"] },
        Property { name: "facing", values: &["north", "south", "west", "east"] },
    ];
    const ITEM: Option<u16> = Some(508);
    const WALL_VARIANT: Option<BlockId> = None;
    const SETTINGS: BlockSettings = BlockSettings {
        hardness: -1.0,
        resistance: 3600000.0,
        friction: 0.6,
        speed_factor: 1.0,
        jump_factor: 1.0,
        bounce_restitution: 0.0,
        fall_distance_reduction: 0.0,
        requires_correct_tool: false,
        ignited_by_lava: false,
    };
    const LIGHT_EMISSION: PerState<u8> = PerState::Uniform(1);
    const LIGHT_DAMPENING: PerState<u8> = PerState::Uniform(0);
    const FLAGS: PerState<StateFlags> = PerState::Uniform(StateFlags(46));
    const COLLISION: PerState<ShapeId> =
        PerState::Varies(crate::shapes::COLLISION_END_PORTAL_FRAME);
    const FACE_SUPPORT: PerState<FaceSupport> =
        PerState::Varies(crate::shapes::SUPPORT_END_PORTAL_FRAME);
    type Behavior = crate::family::HorizontalAway;
}
