use crate::{
    Block, BlockId, BlockSettings, FaceSupport, PerState, Property, ShapeId, StateFlags, StateId,
};

pub struct PaleMossCarpet;

impl Block for PaleMossCarpet {
    const NAME: &'static str = "minecraft:pale_moss_carpet";
    const ID: BlockId = BlockId(1279);
    const FIRST_STATE: StateId = StateId(35554);
    const STATE_COUNT: u16 = 162;
    const DEFAULT_STATE: StateId = StateId(35554);
    const PROPERTIES: &'static [Property] = &[
        Property { name: "bottom", values: &["true", "false"] },
        Property { name: "east", values: &["none", "low", "tall"] },
        Property { name: "north", values: &["none", "low", "tall"] },
        Property { name: "south", values: &["none", "low", "tall"] },
        Property { name: "west", values: &["none", "low", "tall"] },
    ];
    const ITEM: Option<u16> = Some(334);
    const WALL_VARIANT: Option<BlockId> = None;
    const SETTINGS: BlockSettings = BlockSettings {
        hardness: 0.1,
        resistance: 0.1,
        friction: 0.6,
        speed_factor: 1.0,
        jump_factor: 1.0,
        requires_correct_tool: false,
        ignited_by_lava: true,
    };
    const LIGHT_EMISSION: PerState<u8> = PerState::Uniform(0);
    const LIGHT_DAMPENING: PerState<u8> = PerState::Uniform(0);
    const FLAGS: PerState<StateFlags> = PerState::Varies(&[
        StateFlags(36), StateFlags(36), StateFlags(36), StateFlags(36), StateFlags(36),
        StateFlags(36), StateFlags(36), StateFlags(36), StateFlags(36), StateFlags(36),
        StateFlags(36), StateFlags(36), StateFlags(36), StateFlags(36), StateFlags(36),
        StateFlags(36), StateFlags(36), StateFlags(36), StateFlags(36), StateFlags(36),
        StateFlags(36), StateFlags(36), StateFlags(36), StateFlags(36), StateFlags(36),
        StateFlags(36), StateFlags(36), StateFlags(36), StateFlags(36), StateFlags(36),
        StateFlags(36), StateFlags(36), StateFlags(36), StateFlags(36), StateFlags(36),
        StateFlags(36), StateFlags(36), StateFlags(36), StateFlags(36), StateFlags(36),
        StateFlags(36), StateFlags(36), StateFlags(36), StateFlags(36), StateFlags(36),
        StateFlags(36), StateFlags(36), StateFlags(36), StateFlags(36), StateFlags(36),
        StateFlags(36), StateFlags(36), StateFlags(36), StateFlags(36), StateFlags(36),
        StateFlags(36), StateFlags(36), StateFlags(36), StateFlags(36), StateFlags(36),
        StateFlags(36), StateFlags(36), StateFlags(36), StateFlags(36), StateFlags(36),
        StateFlags(36), StateFlags(36), StateFlags(36), StateFlags(36), StateFlags(36),
        StateFlags(36), StateFlags(36), StateFlags(36), StateFlags(36), StateFlags(36),
        StateFlags(36), StateFlags(36), StateFlags(36), StateFlags(36), StateFlags(36),
        StateFlags(36), StateFlags(32), StateFlags(32), StateFlags(32), StateFlags(32),
        StateFlags(32), StateFlags(32), StateFlags(32), StateFlags(32), StateFlags(32),
        StateFlags(32), StateFlags(32), StateFlags(32), StateFlags(32), StateFlags(32),
        StateFlags(32), StateFlags(32), StateFlags(32), StateFlags(32), StateFlags(32),
        StateFlags(32), StateFlags(32), StateFlags(32), StateFlags(32), StateFlags(32),
        StateFlags(32), StateFlags(32), StateFlags(32), StateFlags(32), StateFlags(32),
        StateFlags(32), StateFlags(32), StateFlags(32), StateFlags(32), StateFlags(32),
        StateFlags(32), StateFlags(32), StateFlags(32), StateFlags(32), StateFlags(32),
        StateFlags(32), StateFlags(32), StateFlags(32), StateFlags(32), StateFlags(32),
        StateFlags(32), StateFlags(32), StateFlags(32), StateFlags(32), StateFlags(32),
        StateFlags(32), StateFlags(32), StateFlags(32), StateFlags(32), StateFlags(32),
        StateFlags(32), StateFlags(32), StateFlags(32), StateFlags(32), StateFlags(32),
        StateFlags(32), StateFlags(32), StateFlags(32), StateFlags(32), StateFlags(32),
        StateFlags(32), StateFlags(32), StateFlags(32), StateFlags(32), StateFlags(32),
        StateFlags(32), StateFlags(32), StateFlags(32), StateFlags(32), StateFlags(32),
        StateFlags(32), StateFlags(32), StateFlags(32), StateFlags(32), StateFlags(32),
        StateFlags(32), StateFlags(32),
    ]);
    const COLLISION: PerState<ShapeId> =
        PerState::Varies(crate::shapes::COLLISION_PALE_MOSS_CARPET);
    const FACE_SUPPORT: PerState<FaceSupport> =
        PerState::Varies(crate::shapes::SUPPORT_PALE_MOSS_CARPET);
    type Behavior = crate::family::Plain;
}
