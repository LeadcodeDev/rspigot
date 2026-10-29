use crate::codec::{Decode, DecodeError, Encode};
use crate::ids::play::clientbound as id;
use crate::play::slot::{ItemStack, ItemStackTemplate};
use crate::play::types::HolderSet;
use crate::play::wire::{List, Plain, UNBOUNDED, Var, read_var, wire_struct, write_var};
use crate::types::Text;

wire_struct! {
    #[derive(Debug, Clone, PartialEq, Eq)]
    pub struct ContainerSetContent = id::CONTAINER_SET_CONTENT {
        pub container_id: i32 as Var,
        pub state_id: i32 as Var,
        pub items: Vec<ItemStack> as List<Plain, UNBOUNDED>,
        pub carried: ItemStack,
    }
}

wire_struct! {
    #[derive(Debug, Clone, PartialEq, Eq)]
    pub struct ContainerSetSlot = id::CONTAINER_SET_SLOT {
        pub container_id: i32 as Var,
        pub state_id: i32 as Var,
        pub slot: i16,
        pub item: ItemStack,
    }
}

wire_struct! {
    #[derive(Debug, Clone, PartialEq, Eq)]
    pub struct SetCursorItem = id::SET_CURSOR_ITEM {
        pub item: ItemStack,
    }
}

wire_struct! {
    #[derive(Debug, Clone, PartialEq, Eq)]
    pub struct SetPlayerInventory = id::SET_PLAYER_INVENTORY {
        pub slot: i32 as Var,
        pub item: ItemStack,
    }
}

wire_struct! {
    #[derive(Debug, Clone, PartialEq)]
    pub struct OpenScreen = id::OPEN_SCREEN {
        pub container_id: i32 as Var,
        pub menu: i32 as Var,
        pub title: Text,
    }
}

wire_struct! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub struct ContainerClose = id::CONTAINER_CLOSE {
        pub container_id: i32 as Var,
    }
}

pub mod slot_display {
    pub const EMPTY: i32 = 0;
    pub const ANY_FUEL: i32 = 1;
    pub const WITH_ANY_POTION: i32 = 2;
    pub const ONLY_WITH_COMPONENT: i32 = 3;
    pub const ITEM: i32 = 4;
    pub const ITEM_STACK: i32 = 5;
    pub const TAG: i32 = 6;
    pub const DYED: i32 = 7;
    pub const SMITHING_TRIM: i32 = 8;
    pub const WITH_REMAINDER: i32 = 9;
    pub const COMPOSITE: i32 = 10;
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SlotDisplay {
    Empty,
    AnyFuel,
    Item(i32),
    ItemStack(ItemStackTemplate),
    Tag(HolderSet),
}

impl Encode for SlotDisplay {
    fn encode(&self, out: &mut Vec<u8>) {
        match self {
            Self::Empty => write_var(slot_display::EMPTY, out),
            Self::AnyFuel => write_var(slot_display::ANY_FUEL, out),
            Self::Item(item) => {
                write_var(slot_display::ITEM, out);
                write_var(*item, out);
            }
            Self::ItemStack(template) => {
                write_var(slot_display::ITEM_STACK, out);
                template.encode(out);
            }
            Self::Tag(set) => {
                write_var(slot_display::TAG, out);
                set.encode(out);
            }
        }
    }
}

impl Decode for SlotDisplay {
    fn decode(input: &mut &[u8]) -> Result<Self, DecodeError> {
        Ok(match read_var(input)? {
            slot_display::EMPTY => Self::Empty,
            slot_display::ANY_FUEL => Self::AnyFuel,
            slot_display::ITEM => Self::Item(read_var(input)?),
            slot_display::ITEM_STACK => Self::ItemStack(ItemStackTemplate::decode(input)?),
            slot_display::TAG => Self::Tag(HolderSet::decode(input)?),
            other => {
                return Err(DecodeError::InvalidValue {
                    field: "slot display type",
                    value: i64::from(other),
                });
            }
        })
    }
}

wire_struct! {
    #[derive(Debug, Clone, PartialEq, Eq)]
    pub struct RecipePropertySet {
        pub key: String,
        pub items: Vec<i32> as List<Var, UNBOUNDED>,
    }
}

wire_struct! {
    #[derive(Debug, Clone, PartialEq, Eq)]
    pub struct StonecutterEntry {
        pub input: HolderSet,
        pub display: SlotDisplay,
    }
}

wire_struct! {
    #[derive(Debug, Clone, PartialEq, Eq)]
    pub struct UpdateRecipes = id::UPDATE_RECIPES {
        pub property_sets: Vec<RecipePropertySet> as List<Plain, UNBOUNDED>,
        pub stonecutter: Vec<StonecutterEntry> as List<Plain, UNBOUNDED>,
    }
}

wire_struct! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
    pub struct RecipeBookTypeSettings {
        pub open: bool,
        pub filtering: bool,
    }
}

wire_struct! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
    pub struct RecipeBookSettings = id::RECIPE_BOOK_SETTINGS {
        pub crafting: RecipeBookTypeSettings,
        pub furnace: RecipeBookTypeSettings,
        pub blast_furnace: RecipeBookTypeSettings,
        pub smoker: RecipeBookTypeSettings,
    }
}

#[cfg(test)]
mod tests {
    use rspigot_nbt::Tag;

    use super::*;
    use crate::codec::decode_exact;
    use crate::play::slot::DataComponentPatch;
    use crate::play::wire::testing::roundtrip;

    #[test]
    fn container_content_layout() {
        let packet = ContainerSetContent {
            container_id: 0,
            state_id: 3,
            items: vec![ItemStack::EMPTY, ItemStack::new(1, 64)],
            carried: ItemStack::EMPTY,
        };
        assert_eq!(roundtrip(&packet), vec![0, 3, 2, 0, 64, 1, 0, 0, 0]);
        let slot = ContainerSetSlot {
            container_id: -1,
            state_id: 0,
            slot: 36,
            item: ItemStack::new(1, 1),
        };
        assert_eq!(
            roundtrip(&slot),
            vec![0xFF, 0xFF, 0xFF, 0xFF, 0x0F, 0, 0, 36, 1, 1, 0, 0]
        );
    }

    #[test]
    fn single_item_packets() {
        assert_eq!(
            roundtrip(&SetCursorItem {
                item: ItemStack::EMPTY
            }),
            vec![0]
        );
        assert_eq!(
            roundtrip(&SetPlayerInventory {
                slot: 40,
                item: ItemStack::new(1, 2)
            }),
            vec![40, 2, 1, 0, 0]
        );
        assert_eq!(roundtrip(&ContainerClose { container_id: 1 }), vec![1]);
        let screen = OpenScreen {
            container_id: 1,
            menu: 2,
            title: Text(Tag::String("Chest".into())),
        };
        assert_eq!(
            roundtrip(&screen),
            vec![1, 2, 8, 0, 5, b'C', b'h', b'e', b's', b't']
        );
    }

    #[test]
    fn recipes_layout() {
        let packet = UpdateRecipes {
            property_sets: vec![RecipePropertySet {
                key: "minecraft:furnace_input".into(),
                items: vec![1, 2],
            }],
            stonecutter: vec![StonecutterEntry {
                input: HolderSet::Ids(vec![1]),
                display: SlotDisplay::ItemStack(ItemStackTemplate {
                    item: 3,
                    count: 2,
                    components: DataComponentPatch::EMPTY,
                }),
            }],
        };
        let bytes = roundtrip(&packet);
        assert_eq!(&bytes[..2], &[1, 23]);
        assert_eq!(&bytes[25..28], &[2, 1, 2]);
        assert_eq!(&bytes[28..], &[1, 2, 1, 5, 3, 2, 0, 0]);
        for display in [
            SlotDisplay::Empty,
            SlotDisplay::AnyFuel,
            SlotDisplay::Item(5),
            SlotDisplay::Tag(HolderSet::Tag("minecraft:planks".into())),
        ] {
            roundtrip(&display);
        }
        assert!(decode_exact::<SlotDisplay>(&[10, 0]).is_err());
    }

    #[test]
    fn recipe_book_settings_are_eight_booleans() {
        let settings = RecipeBookSettings {
            crafting: RecipeBookTypeSettings {
                open: true,
                filtering: false,
            },
            smoker: RecipeBookTypeSettings {
                open: false,
                filtering: true,
            },
            ..RecipeBookSettings::default()
        };
        assert_eq!(roundtrip(&settings), vec![1, 0, 0, 0, 0, 0, 0, 1]);
    }
}
