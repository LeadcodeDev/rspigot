use crate::{Block, BlockId, BlockSettings, PerState, Property, StateFlags, StateId};

pub struct WaxedCopperDoor;

impl Block for WaxedCopperDoor {
    const NAME: &'static str = "minecraft:waxed_copper_door";
    const ID: BlockId = BlockId(1170);
    const FIRST_STATE: StateId = StateId(32109);
    const STATE_COUNT: u16 = 64;
    const DEFAULT_STATE: StateId = StateId(32120);
    const PROPERTIES: &'static [Property] = &[
        Property { name: "facing", values: &["north", "south", "west", "east"] },
        Property { name: "half", values: &["upper", "lower"] },
        Property { name: "hinge", values: &["left", "right"] },
        Property { name: "open", values: &["true", "false"] },
        Property { name: "powered", values: &["true", "false"] },
    ];
    const ITEM: Option<u16> = Some(906);
    const WALL_VARIANT: Option<BlockId> = None;
    const SETTINGS: BlockSettings = BlockSettings {
        hardness: 3.0,
        resistance: 6.0,
        friction: 0.6,
        speed_factor: 1.0,
        jump_factor: 1.0,
        requires_correct_tool: false,
        ignited_by_lava: false,
    };
    const LIGHT_EMISSION: PerState<u8> = PerState::Uniform(0);
    const LIGHT_DAMPENING: PerState<u8> = PerState::Uniform(0);
    const FLAGS: PerState<StateFlags> = PerState::Uniform(StateFlags(38));
    type Behavior = crate::family::Door<true>;
}
