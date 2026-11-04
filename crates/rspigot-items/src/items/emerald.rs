use rspigot_blocks::BlockId;

use crate::{
    AttributeModifier, CookingKind, Craft, EquipmentSlot, Food, Item, ItemId, ItemSet, Product,
    Rarity, Tool,
};

pub struct Emerald;

impl Item for Emerald {
    const NAME: &'static str = "minecraft:emerald";
    const ID: ItemId = ItemId(1013);
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
    const TRIM_MATERIAL: Option<&'static str> = Some("minecraft:emerald");
    const CRAFTS: &'static [Craft] = &[
        Craft::Shapeless {
            id: "minecraft:emerald",
            ingredients: &[ItemSet::Direct(&[ItemId(514)])],
            product: Product::of(9),
        },
        Craft::Cooking {
            id: "minecraft:emerald_from_blasting_deepslate_emerald_ore",
            kind: CookingKind::Blasting,
            ingredient: ItemSet::Direct(&[ItemId(104)]),
            experience: 1.0,
            cooking_time: 200,
        },
        Craft::Cooking {
            id: "minecraft:emerald_from_blasting_emerald_ore",
            kind: CookingKind::Blasting,
            ingredient: ItemSet::Direct(&[ItemId(103)]),
            experience: 1.0,
            cooking_time: 200,
        },
        Craft::Cooking {
            id: "minecraft:emerald_from_smelting_deepslate_emerald_ore",
            kind: CookingKind::Smelting,
            ingredient: ItemSet::Direct(&[ItemId(104)]),
            experience: 1.0,
            cooking_time: 200,
        },
        Craft::Cooking {
            id: "minecraft:emerald_from_smelting_emerald_ore",
            kind: CookingKind::Smelting,
            ingredient: ItemSet::Direct(&[ItemId(103)]),
            experience: 1.0,
            cooking_time: 200,
        },
    ];
    type Behavior = crate::family::Plain;
}
