use crate::{
    Block, BlockId, BlockSettings, FaceSupport, PerState, Property, ShapeId, StateFlags, StateId,
};

pub struct NoteBlock;

impl Block for NoteBlock {
    const NAME: &'static str = "minecraft:note_block";
    const ID: BlockId = BlockId(118);
    const FIRST_STATE: StateId = StateId(680);
    const STATE_COUNT: u16 = 1350;
    const DEFAULT_STATE: StateId = StateId(681);
    const PROPERTIES: &'static [Property] = &[
        Property {
            name: "instrument",
            values: &[
                "harp", "basedrum", "snare", "hat", "bass", "flute", "bell", "guitar", "chime",
                "xylophone", "iron_xylophone", "cow_bell", "didgeridoo", "bit", "banjo", "pling",
                "trumpet", "trumpet_exposed", "trumpet_oxidized", "trumpet_weathered", "zombie",
                "skeleton", "creeper", "dragon", "wither_skeleton", "piglin", "custom_head",
            ],
        },
        Property {
            name: "note",
            values: &[
                "0", "1", "2", "3", "4", "5", "6", "7", "8", "9", "10", "11", "12", "13", "14",
                "15", "16", "17", "18", "19", "20", "21", "22", "23", "24",
            ],
        },
        Property { name: "powered", values: &["true", "false"] },
    ];
    const ITEM: Option<u16> = Some(855);
    const WALL_VARIANT: Option<BlockId> = None;
    const SETTINGS: BlockSettings = BlockSettings {
        hardness: 0.8,
        resistance: 0.8,
        friction: 0.6,
        speed_factor: 1.0,
        jump_factor: 1.0,
        requires_correct_tool: false,
        ignited_by_lava: true,
    };
    const LIGHT_EMISSION: PerState<u8> = PerState::Uniform(0);
    const LIGHT_DAMPENING: PerState<u8> = PerState::Uniform(15);
    const FLAGS: PerState<StateFlags> = PerState::Uniform(StateFlags(1054));
    const COLLISION: PerState<ShapeId> = PerState::Uniform(ShapeId(1));
    const FACE_SUPPORT: PerState<FaceSupport> = PerState::Uniform(FaceSupport(63, 63, 63));
    type Behavior = crate::family::Plain;
}
