use crate::{
    Block, BlockId, BlockSettings, FaceSupport, PerState, Property, ShapeId, StateFlags, StateId,
};

pub struct BlastFurnace;

impl Block for BlastFurnace {
    const NAME: &'static str = "minecraft:blast_furnace";
    const ID: BlockId = BlockId(931);
    const FIRST_STATE: StateId = StateId(24119);
    const STATE_COUNT: u16 = 8;
    const DEFAULT_STATE: StateId = StateId(24120);
    const PROPERTIES: &'static [Property] = &[
        Property { name: "facing", values: &["north", "south", "west", "east"] },
        Property { name: "lit", values: &["true", "false"] },
    ];
    const ITEM: Option<u16> = Some(1508);
    const WALL_VARIANT: Option<BlockId> = None;
    const SETTINGS: BlockSettings = BlockSettings {
        hardness: 3.5,
        resistance: 3.5,
        friction: 0.6,
        speed_factor: 1.0,
        jump_factor: 1.0,
        requires_correct_tool: true,
        ignited_by_lava: false,
    };
    const LIGHT_EMISSION: PerState<u8> = PerState::Varies(&[
        13, 0, 13, 0, 13, 0, 13, 0,
    ]);
    const LIGHT_DAMPENING: PerState<u8> = PerState::Uniform(15);
    const FLAGS: PerState<StateFlags> = PerState::Uniform(StateFlags(1054));
    const COLLISION: PerState<ShapeId> = PerState::Uniform(ShapeId(1));
    const FACE_SUPPORT: PerState<FaceSupport> = PerState::Uniform(FaceSupport(63, 63, 63));
    type Behavior = crate::family::HorizontalAway;
}
