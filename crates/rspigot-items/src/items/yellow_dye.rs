use rspigot_blocks::BlockId;

use crate::{
    AttributeModifier, Craft, EquipmentSlot, Food, Item, ItemId, ItemSet, Product, Rarity, Tool,
};

pub struct YellowDye;

impl Item for YellowDye {
    const NAME: &'static str = "minecraft:yellow_dye";
    const ID: ItemId = ItemId(1203);
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
            id: "minecraft:yellow_dye_from_dandelion",
            ingredients: &[ItemSet::Direct(&[ItemId(298)])],
            product: Product::of(1),
        },
        Craft::Shapeless {
            id: "minecraft:yellow_dye_from_golden_dandelion",
            ingredients: &[ItemSet::Direct(&[ItemId(299)])],
            product: Product::of(1),
        },
        Craft::Shapeless {
            id: "minecraft:yellow_dye_from_sunflower",
            ingredients: &[ItemSet::Direct(&[ItemId(599)])],
            product: Product::of(2),
        },
        Craft::Shapeless {
            id: "minecraft:yellow_dye_from_wildflowers",
            ingredients: &[ItemSet::Direct(&[ItemId(330)])],
            product: Product::of(1),
        },
    ];
    type Behavior = crate::family::Plain;
}
