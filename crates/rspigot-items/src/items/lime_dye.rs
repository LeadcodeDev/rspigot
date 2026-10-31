use rspigot_blocks::BlockId;

use crate::{
    AttributeModifier, CookingKind, Craft, EquipmentSlot, Food, Item, ItemId, ItemSet, Product,
    Rarity, Tool,
};

pub struct LimeDye;

impl Item for LimeDye {
    const NAME: &'static str = "minecraft:lime_dye";
    const ID: ItemId = ItemId(1204);
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
            ingredients: &[ItemSet::Direct(&[ItemId(1212)]), ItemSet::Direct(&[ItemId(1199)])],
            product: Product::of(2),
        },
        Craft::Cooking {
            id: "minecraft:lime_dye_from_smelting",
            kind: CookingKind::Smelting,
            ingredient: ItemSet::Direct(&[ItemId(249)]),
            experience: 0.1,
            cooking_time: 200,
        },
    ];
    type Behavior = crate::family::Plain;
}
