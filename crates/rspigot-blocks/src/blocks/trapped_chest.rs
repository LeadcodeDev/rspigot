use crate::{
    Block, BlockId, BlockSettings, FaceSupport, PerState, Property, ShapeId, StateFlags, StateId,
};

pub struct TrappedChest;

impl Block for TrappedChest {
    const NAME: &'static str = "minecraft:trapped_chest";
    const ID: BlockId = BlockId(523);
    const FIRST_STATE: StateId = StateId(12974);
    const STATE_COUNT: u16 = 24;
    const DEFAULT_STATE: StateId = StateId(12975);
    const PROPERTIES: &'static [Property] = &[
        Property { name: "facing", values: &["north", "south", "west", "east"] },
        Property { name: "type", values: &["single", "left", "right"] },
        Property { name: "waterlogged", values: &["true", "false"] },
    ];
    const ITEM: Option<u16> = Some(852);
    const WALL_VARIANT: Option<BlockId> = None;
    const SETTINGS: BlockSettings = BlockSettings {
        hardness: 2.5,
        resistance: 2.5,
        friction: 0.6,
        speed_factor: 1.0,
        jump_factor: 1.0,
        requires_correct_tool: false,
        ignited_by_lava: true,
    };
    const LIGHT_EMISSION: PerState<u8> = PerState::Uniform(0);
    const LIGHT_DAMPENING: PerState<u8> = PerState::Varies(&[
        1, 0, 1, 0, 1, 0, 1, 0, 1, 0, 1, 0, 1, 0, 1, 0, 1, 0, 1, 0, 1, 0, 1, 0,
    ]);
    const FLAGS: PerState<StateFlags> = PerState::Varies(&[
        StateFlags(142), StateFlags(46), StateFlags(142), StateFlags(46), StateFlags(142),
        StateFlags(46), StateFlags(142), StateFlags(46), StateFlags(142), StateFlags(46),
        StateFlags(142), StateFlags(46), StateFlags(142), StateFlags(46), StateFlags(142),
        StateFlags(46), StateFlags(142), StateFlags(46), StateFlags(142), StateFlags(46),
        StateFlags(142), StateFlags(46), StateFlags(142), StateFlags(46),
    ]);
    const COLLISION: PerState<ShapeId> = PerState::Varies(crate::shapes::COLLISION_CHEST);
    const FACE_SUPPORT: PerState<FaceSupport> = PerState::Varies(crate::shapes::SUPPORT_CHEST);
    type Behavior = crate::family::HorizontalAway;
}
