use crate::{
    Block, BlockId, BlockSettings, FaceSupport, PerState, Property, ShapeId, StateFlags, StateId,
};

pub struct Vault;

impl Block for Vault {
    const NAME: &'static str = "minecraft:vault";
    const ID: BlockId = BlockId(1276);
    const FIRST_STATE: StateId = StateId(35519);
    const STATE_COUNT: u16 = 32;
    const DEFAULT_STATE: StateId = StateId(35523);
    const PROPERTIES: &'static [Property] = &[
        Property { name: "facing", values: &["north", "south", "west", "east"] },
        Property { name: "ominous", values: &["true", "false"] },
        Property { name: "vault_state", values: &["inactive", "active", "unlocking", "ejecting"] },
    ];
    const ITEM: Option<u16> = Some(1656);
    const WALL_VARIANT: Option<BlockId> = None;
    const SETTINGS: BlockSettings = BlockSettings {
        hardness: 50.0,
        resistance: 50.0,
        friction: 0.6,
        speed_factor: 1.0,
        jump_factor: 1.0,
        bounce_restitution: 0.0,
        fall_distance_reduction: 0.0,
        requires_correct_tool: false,
        ignited_by_lava: false,
    };
    const LIGHT_EMISSION: PerState<u8> = PerState::Varies(&[
        6, 12, 12, 12, 6, 12, 12, 12, 6, 12, 12, 12, 6, 12, 12, 12, 6, 12, 12, 12, 6, 12, 12, 12, 6,
        12, 12, 12, 6, 12, 12, 12,
    ]);
    const LIGHT_DAMPENING: PerState<u8> = PerState::Uniform(1);
    const FLAGS: PerState<StateFlags> = PerState::Uniform(StateFlags(7174));
    const COLLISION: PerState<ShapeId> = PerState::Uniform(ShapeId(1));
    const FACE_SUPPORT: PerState<FaceSupport> = PerState::Uniform(FaceSupport(63, 63, 63));
    type Behavior = crate::family::HorizontalAway;
}
