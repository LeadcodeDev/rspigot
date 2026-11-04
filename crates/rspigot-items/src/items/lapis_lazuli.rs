use rspigot_blocks::BlockId;

use crate::{
    AttributeModifier, CookingKind, Craft, EquipmentSlot, Food, Item, ItemId, ItemSet, Product,
    Rarity, Tool,
};

pub struct LapisLazuli;

impl Item for LapisLazuli {
    const NAME: &'static str = "minecraft:lapis_lazuli";
    const ID: ItemId = ItemId(1014);
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
    const TRIM_MATERIAL: Option<&'static str> = Some("minecraft:lapis");
    const CRAFTS: &'static [Craft] = &[
        Craft::Shapeless {
            id: "minecraft:lapis_lazuli",
            ingredients: &[ItemSet::Direct(&[ItemId(233)])],
            product: Product::of(9),
        },
        Craft::Cooking {
            id: "minecraft:lapis_lazuli_from_blasting_deepslate_lapis_ore",
            kind: CookingKind::Blasting,
            ingredient: ItemSet::Direct(&[ItemId(106)]),
            experience: 0.2,
            cooking_time: 200,
        },
        Craft::Cooking {
            id: "minecraft:lapis_lazuli_from_blasting_lapis_ore",
            kind: CookingKind::Blasting,
            ingredient: ItemSet::Direct(&[ItemId(105)]),
            experience: 0.2,
            cooking_time: 200,
        },
        Craft::Cooking {
            id: "minecraft:lapis_lazuli_from_smelting_deepslate_lapis_ore",
            kind: CookingKind::Smelting,
            ingredient: ItemSet::Direct(&[ItemId(106)]),
            experience: 0.2,
            cooking_time: 200,
        },
        Craft::Cooking {
            id: "minecraft:lapis_lazuli_from_smelting_lapis_ore",
            kind: CookingKind::Smelting,
            ingredient: ItemSet::Direct(&[ItemId(105)]),
            experience: 0.2,
            cooking_time: 200,
        },
    ];
    type Behavior = crate::family::Plain;
}
