use rspigot_blocks::BlockId;

use crate::{
    AttributeModifier, Craft, EquipmentSlot, Food, Item, ItemId, ItemSet, Product, Rarity, Tool,
};

pub struct MagentaDye;

impl Item for MagentaDye {
    const NAME: &'static str = "minecraft:magenta_dye";
    const ID: ItemId = ItemId(1201);
    const CLASS: &'static str = "DyeItem";
    const MAX_STACK_SIZE: u8 = 64;
    const MAX_DAMAGE: Option<u32> = None;
    const WEAPON_DAMAGE: Option<u32> = None;
    const RARITY: Rarity = Rarity::Common;
    const BLOCK: Option<BlockId> = None;
    const TOOL: Option<Tool> = None;
    const FOOD: Option<Food> = None;
    const EQUIPPABLE: Option<EquipmentSlot> = None;
    const ENCHANTABLE: Option<u32> = None;
    const REPAIRABLE: Option<ItemSet> = None;
    const DAMAGE_RESISTANT: Option<&'static str> = None;
    const USE_REMAINDER: Option<ItemId> = None;
    const ATTRIBUTE_MODIFIERS: &'static [AttributeModifier] = &[];
    const CRAFTS: &'static [Craft] = &[
        Craft::Shapeless {
            id: "minecraft:magenta_dye_from_allium",
            ingredients: &[ItemSet::Direct(&[ItemId(304)])],
            product: Product::of(1),
        },
        Craft::Shapeless {
            id: "minecraft:magenta_dye_from_blue_red_pink",
            ingredients: &[
                ItemSet::Direct(&[ItemId(1210)]),
                ItemSet::Direct(&[ItemId(1213)]),
                ItemSet::Direct(&[ItemId(1205)]),
            ],
            product: Product::of(3),
        },
        Craft::Shapeless {
            id: "minecraft:magenta_dye_from_blue_red_white_dye",
            ingredients: &[
                ItemSet::Direct(&[ItemId(1210)]),
                ItemSet::Direct(&[ItemId(1213)]),
                ItemSet::Direct(&[ItemId(1213)]),
                ItemSet::Direct(&[ItemId(1199)]),
            ],
            product: Product::of(4),
        },
        Craft::Shapeless {
            id: "minecraft:magenta_dye_from_lilac",
            ingredients: &[ItemSet::Direct(&[ItemId(600)])],
            product: Product::of(2),
        },
        Craft::Shapeless {
            id: "minecraft:magenta_dye_from_purple_and_pink",
            ingredients: &[ItemSet::Direct(&[ItemId(1209)]), ItemSet::Direct(&[ItemId(1205)])],
            product: Product::of(2),
        },
    ];
    type Behavior = crate::family::Plain;
}
