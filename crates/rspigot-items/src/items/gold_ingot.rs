use rspigot_blocks::BlockId;

use crate::{
    AttributeModifier, CookingKind, Craft, EquipmentSlot, Food, Item, ItemId, ItemSet, Product,
    Rarity, Tool,
};

pub struct GoldIngot;

impl Item for GoldIngot {
    const NAME: &'static str = "minecraft:gold_ingot";
    const ID: ItemId = ItemId(1022);
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
    const CRAFTS: &'static [Craft] = &[
        Craft::Cooking {
            id: "minecraft:gold_ingot_from_blasting_deepslate_gold_ore",
            kind: CookingKind::Blasting,
            ingredient: ItemSet::Direct(&[ItemId(100)]),
            experience: 1.0,
            cooking_time: 200,
        },
        Craft::Cooking {
            id: "minecraft:gold_ingot_from_blasting_gold_ore",
            kind: CookingKind::Blasting,
            ingredient: ItemSet::Direct(&[ItemId(99)]),
            experience: 1.0,
            cooking_time: 200,
        },
        Craft::Cooking {
            id: "minecraft:gold_ingot_from_blasting_nether_gold_ore",
            kind: CookingKind::Blasting,
            ingredient: ItemSet::Direct(&[ItemId(109)]),
            experience: 1.0,
            cooking_time: 200,
        },
        Craft::Cooking {
            id: "minecraft:gold_ingot_from_blasting_raw_gold",
            kind: CookingKind::Blasting,
            ingredient: ItemSet::Direct(&[ItemId(1021)]),
            experience: 1.0,
            cooking_time: 200,
        },
        Craft::Shapeless {
            ingredients: &[ItemSet::Direct(&[ItemId(128)])],
            product: Product::of(9),
        },
        Craft::Shaped {
            width: 3,
            height: 3,
            symmetrical: true,
            cells: &[
                Some(ItemSet::Direct(&[ItemId(1268)])),
                Some(ItemSet::Direct(&[ItemId(1268)])),
                Some(ItemSet::Direct(&[ItemId(1268)])),
                Some(ItemSet::Direct(&[ItemId(1268)])),
                Some(ItemSet::Direct(&[ItemId(1268)])),
                Some(ItemSet::Direct(&[ItemId(1268)])),
                Some(ItemSet::Direct(&[ItemId(1268)])),
                Some(ItemSet::Direct(&[ItemId(1268)])),
                Some(ItemSet::Direct(&[ItemId(1268)])),
            ],
            product: Product::of(1),
        },
        Craft::Cooking {
            id: "minecraft:gold_ingot_from_smelting_deepslate_gold_ore",
            kind: CookingKind::Smelting,
            ingredient: ItemSet::Direct(&[ItemId(100)]),
            experience: 1.0,
            cooking_time: 200,
        },
        Craft::Cooking {
            id: "minecraft:gold_ingot_from_smelting_gold_ore",
            kind: CookingKind::Smelting,
            ingredient: ItemSet::Direct(&[ItemId(99)]),
            experience: 1.0,
            cooking_time: 200,
        },
        Craft::Cooking {
            id: "minecraft:gold_ingot_from_smelting_nether_gold_ore",
            kind: CookingKind::Smelting,
            ingredient: ItemSet::Direct(&[ItemId(109)]),
            experience: 1.0,
            cooking_time: 200,
        },
        Craft::Cooking {
            id: "minecraft:gold_ingot_from_smelting_raw_gold",
            kind: CookingKind::Smelting,
            ingredient: ItemSet::Direct(&[ItemId(1021)]),
            experience: 1.0,
            cooking_time: 200,
        },
    ];
    type Behavior = crate::family::Plain;
}
