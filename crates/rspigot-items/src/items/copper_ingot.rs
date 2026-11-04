use rspigot_blocks::BlockId;

use crate::{
    AttributeModifier, CookingKind, Craft, EquipmentSlot, Food, Item, ItemId, ItemSet, Product,
    Rarity, Tool,
};

pub struct CopperIngot;

impl Item for CopperIngot {
    const NAME: &'static str = "minecraft:copper_ingot";
    const ID: ItemId = ItemId(1020);
    const CLASS: &'static str = "Item";
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
    const TRIM_MATERIAL: Option<&'static str> = Some("minecraft:copper");
    const CRAFTS: &'static [Craft] = &[
        Craft::Shapeless {
            id: "minecraft:copper_ingot",
            ingredients: &[ItemSet::Direct(&[ItemId(120)])],
            product: Product::of(9),
        },
        Craft::Cooking {
            id: "minecraft:copper_ingot_from_blasting_copper_ore",
            kind: CookingKind::Blasting,
            ingredient: ItemSet::Direct(&[ItemId(97)]),
            experience: 0.7,
            cooking_time: 200,
        },
        Craft::Cooking {
            id: "minecraft:copper_ingot_from_blasting_deepslate_copper_ore",
            kind: CookingKind::Blasting,
            ingredient: ItemSet::Direct(&[ItemId(98)]),
            experience: 0.7,
            cooking_time: 200,
        },
        Craft::Cooking {
            id: "minecraft:copper_ingot_from_blasting_raw_copper",
            kind: CookingKind::Blasting,
            ingredient: ItemSet::Direct(&[ItemId(1019)]),
            experience: 0.7,
            cooking_time: 200,
        },
        Craft::Shaped {
            id: "minecraft:copper_ingot_from_nuggets",
            width: 3,
            height: 3,
            symmetrical: true,
            cells: &[
                Some(ItemSet::Direct(&[ItemId(1457)])),
                Some(ItemSet::Direct(&[ItemId(1457)])),
                Some(ItemSet::Direct(&[ItemId(1457)])),
                Some(ItemSet::Direct(&[ItemId(1457)])),
                Some(ItemSet::Direct(&[ItemId(1457)])),
                Some(ItemSet::Direct(&[ItemId(1457)])),
                Some(ItemSet::Direct(&[ItemId(1457)])),
                Some(ItemSet::Direct(&[ItemId(1457)])),
                Some(ItemSet::Direct(&[ItemId(1457)])),
            ],
            product: Product::of(1),
        },
        Craft::Cooking {
            id: "minecraft:copper_ingot_from_smelting_copper_ore",
            kind: CookingKind::Smelting,
            ingredient: ItemSet::Direct(&[ItemId(97)]),
            experience: 0.7,
            cooking_time: 200,
        },
        Craft::Cooking {
            id: "minecraft:copper_ingot_from_smelting_deepslate_copper_ore",
            kind: CookingKind::Smelting,
            ingredient: ItemSet::Direct(&[ItemId(98)]),
            experience: 0.7,
            cooking_time: 200,
        },
        Craft::Cooking {
            id: "minecraft:copper_ingot_from_smelting_raw_copper",
            kind: CookingKind::Smelting,
            ingredient: ItemSet::Direct(&[ItemId(1019)]),
            experience: 0.7,
            cooking_time: 200,
        },
        Craft::Shapeless {
            id: "minecraft:copper_ingot_from_waxed_copper_block",
            ingredients: &[ItemSet::Direct(&[ItemId(124)])],
            product: Product::of(9),
        },
    ];
    type Behavior = crate::family::Plain;
}
