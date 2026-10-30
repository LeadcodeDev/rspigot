use crate::{
    Block, BlockId, BlockSettings, FaceSupport, PerState, Property, ShapeId, SpawnSurface,
    StateFlags, StateId,
};

pub struct WeatheredCopperTrapdoor;

impl Block for WeatheredCopperTrapdoor {
    const NAME: &'static str = "minecraft:weathered_copper_trapdoor";
    const ID: BlockId = BlockId(1176);
    const FIRST_STATE: StateId = StateId(32493);
    const STATE_COUNT: u16 = 64;
    const DEFAULT_STATE: StateId = StateId(32508);
    const PROPERTIES: &'static [Property] = &[
        Property { name: "facing", values: &["north", "south", "west", "east"] },
        Property { name: "half", values: &["top", "bottom"] },
        Property { name: "open", values: &["true", "false"] },
        Property { name: "powered", values: &["true", "false"] },
        Property { name: "waterlogged", values: &["true", "false"] },
    ];
    const ITEM: Option<u16> = Some(926);
    const WALL_VARIANT: Option<BlockId> = None;
    const SETTINGS: BlockSettings = BlockSettings {
        hardness: 3.0,
        resistance: 6.0,
        friction: 0.6,
        speed_factor: 1.0,
        jump_factor: 1.0,
        bounce_restitution: 0.0,
        fall_distance_reduction: 0.0,
        requires_correct_tool: true,
        ignited_by_lava: false,
    };
    const LIGHT_EMISSION: PerState<u8> = PerState::Uniform(0);
    const LIGHT_DAMPENING: PerState<u8> = PerState::Varies(&[
        1, 0, 1, 0, 1, 0, 1, 0, 1, 0, 1, 0, 1, 0, 1, 0, 1, 0, 1, 0, 1, 0, 1, 0, 1, 0, 1, 0, 1, 0, 1,
        0, 1, 0, 1, 0, 1, 0, 1, 0, 1, 0, 1, 0, 1, 0, 1, 0, 1, 0, 1, 0, 1, 0, 1, 0, 1, 0, 1, 0, 1, 0,
        1, 0,
    ]);
    const FLAGS: PerState<StateFlags> = PerState::Varies(&[
        StateFlags(19078), StateFlags(18982), StateFlags(19078), StateFlags(18982),
        StateFlags(2694), StateFlags(2598), StateFlags(2694), StateFlags(2598), StateFlags(19078),
        StateFlags(18982), StateFlags(19078), StateFlags(18982), StateFlags(2694), StateFlags(2598),
        StateFlags(2694), StateFlags(2598), StateFlags(19078), StateFlags(18982), StateFlags(19078),
        StateFlags(18982), StateFlags(2694), StateFlags(2598), StateFlags(2694), StateFlags(2598),
        StateFlags(19078), StateFlags(18982), StateFlags(19078), StateFlags(18982),
        StateFlags(2694), StateFlags(2598), StateFlags(2694), StateFlags(2598), StateFlags(19078),
        StateFlags(18982), StateFlags(19078), StateFlags(18982), StateFlags(2694), StateFlags(2598),
        StateFlags(2694), StateFlags(2598), StateFlags(19078), StateFlags(18982), StateFlags(19078),
        StateFlags(18982), StateFlags(2694), StateFlags(2598), StateFlags(2694), StateFlags(2598),
        StateFlags(19078), StateFlags(18982), StateFlags(19078), StateFlags(18982),
        StateFlags(2694), StateFlags(2598), StateFlags(2694), StateFlags(2598), StateFlags(19078),
        StateFlags(18982), StateFlags(19078), StateFlags(18982), StateFlags(2694), StateFlags(2598),
        StateFlags(2694), StateFlags(2598),
    ]);
    const COLLISION: PerState<ShapeId> = PerState::Varies(crate::shapes::COLLISION_OAK_TRAPDOOR);
    const FACE_SUPPORT: PerState<FaceSupport> =
        PerState::Varies(crate::shapes::SUPPORT_OAK_TRAPDOOR);
    const SPAWN_SURFACE: SpawnSurface = SpawnSurface::Never;
    type Behavior = crate::family::Trapdoor<true>;
}
