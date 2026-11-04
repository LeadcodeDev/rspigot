use rspigot_blocks::BlockId;

use crate::{
    AttributeModifier, Craft, EquipmentSlot, Food, Item, ItemId, ItemSet, Product, Rarity, Tool,
};

pub struct Sugar;

impl Item for Sugar {
    const NAME: &'static str = "minecraft:sugar";
    const ID: ItemId = ItemId(1217);
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
        Craft::Shapeless {
            id: "minecraft:sugar_from_honey_bottle",
            ingredients: &[ItemSet::Direct(&[ItemId(1533)])],
            product: Product::of(3),
        },
        Craft::Shapeless {
            id: "minecraft:sugar_from_sugar_cane",
            ingredients: &[ItemSet::Direct(&[ItemId(327)])],
            product: Product::of(1),
        },
    ];
    type Behavior = crate::family::Plain;
}
