use crate::{
    Block, BlockId, BlockSettings, FaceSupport, PerState, Property, ShapeId, StateFlags, StateId,
};

pub struct TrialSpawner;

impl Block for TrialSpawner {
    const NAME: &'static str = "minecraft:trial_spawner";
    const ID: BlockId = BlockId(1275);
    const FIRST_STATE: StateId = StateId(35507);
    const STATE_COUNT: u16 = 12;
    const DEFAULT_STATE: StateId = StateId(35513);
    const PROPERTIES: &'static [Property] = &[
        Property { name: "ominous", values: &["true", "false"] },
        Property {
            name: "trial_spawner_state",
            values: &[
                "inactive", "waiting_for_players", "active", "waiting_for_reward_ejection",
                "ejecting_reward", "cooldown",
            ],
        },
    ];
    const ITEM: Option<u16> = Some(1653);
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
        0, 4, 8, 8, 8, 0, 0, 4, 8, 8, 8, 0,
    ]);
    const LIGHT_DAMPENING: PerState<u8> = PerState::Uniform(1);
    const FLAGS: PerState<StateFlags> = PerState::Uniform(StateFlags(3078));
    const COLLISION: PerState<ShapeId> = PerState::Uniform(ShapeId(1));
    const FACE_SUPPORT: PerState<FaceSupport> = PerState::Uniform(FaceSupport(63, 63, 63));
    type Behavior = crate::family::Plain;
}
