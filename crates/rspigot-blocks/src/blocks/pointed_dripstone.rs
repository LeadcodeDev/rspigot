use crate::{
    Block, BlockId, BlockSettings, FaceSupport, PerState, Property, ShapeId, StateFlags, StateId,
};

pub struct PointedDripstone;

impl Block for PointedDripstone {
    const NAME: &'static str = "minecraft:pointed_dripstone";
    const ID: BlockId = BlockId(1223);
    const FIRST_STATE: StateId = StateId(33566);
    const STATE_COUNT: u16 = 20;
    const DEFAULT_STATE: StateId = StateId(33571);
    const PROPERTIES: &'static [Property] = &[
        Property { name: "thickness", values: &["tip_merge", "tip", "frustum", "middle", "base"] },
        Property { name: "vertical_direction", values: &["up", "down"] },
        Property { name: "waterlogged", values: &["true", "false"] },
    ];
    const ITEM: Option<u16> = Some(1571);
    const WALL_VARIANT: Option<BlockId> = None;
    const SETTINGS: BlockSettings = BlockSettings {
        hardness: 1.5,
        resistance: 3.0,
        friction: 0.6,
        speed_factor: 1.0,
        jump_factor: 1.0,
        requires_correct_tool: false,
        ignited_by_lava: false,
    };
    const LIGHT_EMISSION: PerState<u8> = PerState::Uniform(0);
    const LIGHT_DAMPENING: PerState<u8> = PerState::Varies(&[
        1, 0, 1, 0, 1, 0, 1, 0, 1, 0, 1, 0, 1, 0, 1, 0, 1, 0, 1, 0,
    ]);
    const FLAGS: PerState<StateFlags> = PerState::Varies(&[
        StateFlags(646), StateFlags(550), StateFlags(646), StateFlags(550), StateFlags(646),
        StateFlags(550), StateFlags(646), StateFlags(550), StateFlags(646), StateFlags(550),
        StateFlags(646), StateFlags(550), StateFlags(646), StateFlags(550), StateFlags(646),
        StateFlags(550), StateFlags(646), StateFlags(550), StateFlags(646), StateFlags(550),
    ]);
    const COLLISION: PerState<ShapeId> =
        PerState::Varies(crate::shapes::COLLISION_POINTED_DRIPSTONE);
    const FACE_SUPPORT: PerState<FaceSupport> =
        PerState::Varies(crate::shapes::SUPPORT_POINTED_DRIPSTONE);
    type Behavior = crate::family::Speleothem;
}
