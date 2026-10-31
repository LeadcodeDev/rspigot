use rspigot_blocks::BlockId;

use crate::{
    AttributeModifier, CookingKind, Craft, EquipmentSlot, Food, Item, ItemId, ItemSet, Product,
    Rarity, Tool,
};

pub struct Redstone;

impl Item for Redstone {
    const NAME: &'static str = "minecraft:redstone";
    const ID: ItemId = ItemId(824);
    const CLASS: &'static str = "BlockItem";
    const MAX_STACK_SIZE: u8 = 64;
    const MAX_DAMAGE: Option<u32> = None;
    const WEAPON_DAMAGE: Option<u32> = None;
    const RARITY: Rarity = Rarity::Common;
    const BLOCK: Option<BlockId> = Some(BlockId(246));
    const TOOL: Option<Tool> = None;
    const FOOD: Option<Food> = None;
    const EQUIPPABLE: Option<EquipmentSlot> = None;
    const ENCHANTABLE: Option<u32> = None;
    const REPAIRABLE: Option<ItemSet> = None;
    const DAMAGE_RESISTANT: Option<&'static str> = None;
    const USE_REMAINDER: Option<ItemId> = None;
    const ATTRIBUTE_MODIFIERS: &'static [AttributeModifier] = &[];
    const TRIM_MATERIAL: Option<&'static str> = Some("minecraft:redstone");
    const CRAFTS: &'static [Craft] = &[
        Craft::Shapeless {
            ingredients: &[ItemSet::Direct(&[ItemId(826)])],
            product: Product::of(9),
        },
        Craft::Cooking {
            id: "minecraft:redstone_from_blasting_deepslate_redstone_ore",
            kind: CookingKind::Blasting,
            ingredient: ItemSet::Direct(&[ItemId(102)]),
            experience: 0.7,
            cooking_time: 200,
        },
        Craft::Cooking {
            id: "minecraft:redstone_from_blasting_redstone_ore",
            kind: CookingKind::Blasting,
            ingredient: ItemSet::Direct(&[ItemId(101)]),
            experience: 0.7,
            cooking_time: 200,
        },
        Craft::Cooking {
            id: "minecraft:redstone_from_smelting_deepslate_redstone_ore",
            kind: CookingKind::Smelting,
            ingredient: ItemSet::Direct(&[ItemId(102)]),
            experience: 0.7,
            cooking_time: 200,
        },
        Craft::Cooking {
            id: "minecraft:redstone_from_smelting_redstone_ore",
            kind: CookingKind::Smelting,
            ingredient: ItemSet::Direct(&[ItemId(101)]),
            experience: 0.7,
            cooking_time: 200,
        },
    ];
    type Behavior = crate::family::BlockItem;
}
