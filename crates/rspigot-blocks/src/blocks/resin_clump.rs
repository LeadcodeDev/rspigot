use crate::{Block, BlockId, BlockSettings, PerState, Property, StateFlags, StateId};

pub struct ResinClump;

impl Block for ResinClump {
    const NAME: &'static str = "minecraft:resin_clump";
    const ID: BlockId = BlockId(418);
    const FIRST_STATE: StateId = StateId(10252);
    const STATE_COUNT: u16 = 128;
    const DEFAULT_STATE: StateId = StateId(10379);
    const PROPERTIES: &'static [Property] = &[
        Property { name: "down", values: &["true", "false"] },
        Property { name: "east", values: &["true", "false"] },
        Property { name: "north", values: &["true", "false"] },
        Property { name: "south", values: &["true", "false"] },
        Property { name: "up", values: &["true", "false"] },
        Property { name: "waterlogged", values: &["true", "false"] },
        Property { name: "west", values: &["true", "false"] },
    ];
    const ITEM: Option<u16> = Some(486);
    const WALL_VARIANT: Option<BlockId> = None;
    const SETTINGS: BlockSettings = BlockSettings {
        hardness: 0.0,
        resistance: 0.0,
        friction: 0.6,
        speed_factor: 1.0,
        jump_factor: 1.0,
        requires_correct_tool: false,
        ignited_by_lava: true,
    };
    const LIGHT_EMISSION: PerState<u8> = PerState::Uniform(0);
    const LIGHT_DAMPENING: PerState<u8> = PerState::Varies(&[
        1, 1, 0, 0, 1, 1, 0, 0, 1, 1, 0, 0, 1, 1, 0, 0, 1, 1, 0, 0, 1, 1, 0, 0, 1, 1, 0, 0, 1, 1, 0,
        0, 1, 1, 0, 0, 1, 1, 0, 0, 1, 1, 0, 0, 1, 1, 0, 0, 1, 1, 0, 0, 1, 1, 0, 0, 1, 1, 0, 0, 1, 1,
        0, 0, 1, 1, 0, 0, 1, 1, 0, 0, 1, 1, 0, 0, 1, 1, 0, 0, 1, 1, 0, 0, 1, 1, 0, 0, 1, 1, 0, 0, 1,
        1, 0, 0, 1, 1, 0, 0, 1, 1, 0, 0, 1, 1, 0, 0, 1, 1, 0, 0, 1, 1, 0, 0, 1, 1, 0, 0, 1, 1, 0, 0,
        1, 1, 0, 1,
    ]);
    const FLAGS: PerState<StateFlags> = PerState::Varies(&[
        StateFlags(192), StateFlags(192), StateFlags(96), StateFlags(96), StateFlags(192),
        StateFlags(192), StateFlags(96), StateFlags(96), StateFlags(192), StateFlags(192),
        StateFlags(96), StateFlags(96), StateFlags(192), StateFlags(192), StateFlags(96),
        StateFlags(96), StateFlags(192), StateFlags(192), StateFlags(96), StateFlags(96),
        StateFlags(192), StateFlags(192), StateFlags(96), StateFlags(96), StateFlags(192),
        StateFlags(192), StateFlags(96), StateFlags(96), StateFlags(192), StateFlags(192),
        StateFlags(96), StateFlags(96), StateFlags(192), StateFlags(192), StateFlags(96),
        StateFlags(96), StateFlags(192), StateFlags(192), StateFlags(96), StateFlags(96),
        StateFlags(192), StateFlags(192), StateFlags(96), StateFlags(96), StateFlags(192),
        StateFlags(192), StateFlags(96), StateFlags(96), StateFlags(192), StateFlags(192),
        StateFlags(96), StateFlags(96), StateFlags(192), StateFlags(192), StateFlags(96),
        StateFlags(96), StateFlags(192), StateFlags(192), StateFlags(96), StateFlags(96),
        StateFlags(192), StateFlags(192), StateFlags(96), StateFlags(96), StateFlags(192),
        StateFlags(192), StateFlags(96), StateFlags(96), StateFlags(192), StateFlags(192),
        StateFlags(96), StateFlags(96), StateFlags(192), StateFlags(192), StateFlags(96),
        StateFlags(96), StateFlags(192), StateFlags(192), StateFlags(96), StateFlags(96),
        StateFlags(192), StateFlags(192), StateFlags(96), StateFlags(96), StateFlags(192),
        StateFlags(192), StateFlags(96), StateFlags(96), StateFlags(192), StateFlags(192),
        StateFlags(96), StateFlags(96), StateFlags(192), StateFlags(192), StateFlags(96),
        StateFlags(96), StateFlags(192), StateFlags(192), StateFlags(96), StateFlags(96),
        StateFlags(192), StateFlags(192), StateFlags(96), StateFlags(96), StateFlags(192),
        StateFlags(192), StateFlags(96), StateFlags(96), StateFlags(192), StateFlags(192),
        StateFlags(96), StateFlags(96), StateFlags(192), StateFlags(192), StateFlags(96),
        StateFlags(96), StateFlags(192), StateFlags(192), StateFlags(96), StateFlags(96),
        StateFlags(192), StateFlags(192), StateFlags(96), StateFlags(96), StateFlags(192),
        StateFlags(192), StateFlags(96), StateFlags(64),
    ]);
    type Behavior = crate::family::Plain;
}
