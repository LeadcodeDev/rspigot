use crate::{Block, BlockId, BlockSettings, PerState, Property, StateFlags, StateId};

pub struct OxidizedLightningRod;

impl Block for OxidizedLightningRod {
    const NAME: &'static str = "minecraft:oxidized_lightning_rod";
    const ID: BlockId = BlockId(1217);
    const FIRST_STATE: StateId = StateId(33445);
    const STATE_COUNT: u16 = 24;
    const DEFAULT_STATE: StateId = StateId(33464);
    const PROPERTIES: &'static [Property] = &[
        Property { name: "facing", values: &["north", "east", "south", "west", "up", "down"] },
        Property { name: "powered", values: &["true", "false"] },
        Property { name: "waterlogged", values: &["true", "false"] },
    ];
    const ITEM: Option<u16> = Some(843);
    const WALL_VARIANT: Option<BlockId> = None;
    const SETTINGS: BlockSettings = BlockSettings {
        hardness: 3.0,
        resistance: 6.0,
        friction: 0.6,
        speed_factor: 1.0,
        jump_factor: 1.0,
        requires_correct_tool: true,
        ignited_by_lava: false,
    };
    const LIGHT_EMISSION: PerState<u8> = PerState::Uniform(0);
    const LIGHT_DAMPENING: PerState<u8> = PerState::Varies(&[
        1, 0, 1, 0, 1, 0, 1, 0, 1, 0, 1, 0, 1, 0, 1, 0, 1, 0, 1, 0, 1, 0, 1, 0,
    ]);
    const FLAGS: PerState<StateFlags> = PerState::Varies(&[
        StateFlags(134), StateFlags(38), StateFlags(134), StateFlags(38), StateFlags(134),
        StateFlags(38), StateFlags(134), StateFlags(38), StateFlags(134), StateFlags(38),
        StateFlags(134), StateFlags(38), StateFlags(134), StateFlags(38), StateFlags(134),
        StateFlags(38), StateFlags(134), StateFlags(38), StateFlags(134), StateFlags(38),
        StateFlags(134), StateFlags(38), StateFlags(134), StateFlags(38),
    ]);
    type Behavior = crate::family::FaceAttached;
}
