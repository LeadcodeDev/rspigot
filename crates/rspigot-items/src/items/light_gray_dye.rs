use rspigot_blocks::BlockId;

use crate::{
    AttributeModifier, Craft, EquipmentSlot, Food, Item, ItemId, ItemSet, Product, Rarity, Tool,
};

pub struct LightGrayDye;

impl Item for LightGrayDye {
    const NAME: &'static str = "minecraft:light_gray_dye";
    const ID: ItemId = ItemId(1207);
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
            id: "minecraft:light_gray_dye_from_azure_bluet",
            ingredients: &[ItemSet::Direct(&[ItemId(305)])],
            product: Product::of(1),
        },
        Craft::Shapeless {
            id: "minecraft:light_gray_dye_from_black_white_dye",
            ingredients: &[
                ItemSet::Direct(&[ItemId(1214)]),
                ItemSet::Direct(&[ItemId(1199)]),
                ItemSet::Direct(&[ItemId(1199)]),
            ],
            product: Product::of(3),
        },
        Craft::Shapeless {
            id: "minecraft:light_gray_dye_from_gray_white_dye",
            ingredients: &[ItemSet::Direct(&[ItemId(1206)]), ItemSet::Direct(&[ItemId(1199)])],
            product: Product::of(2),
        },
        Craft::Shapeless {
            id: "minecraft:light_gray_dye_from_oxeye_daisy",
            ingredients: &[ItemSet::Direct(&[ItemId(310)])],
            product: Product::of(1),
        },
        Craft::Shapeless {
            id: "minecraft:light_gray_dye_from_white_tulip",
            ingredients: &[ItemSet::Direct(&[ItemId(308)])],
            product: Product::of(1),
        },
    ];
    type Behavior = crate::family::Plain;
}
