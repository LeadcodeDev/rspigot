use rspigot_blocks::BlockId;

use crate::{
    AttributeModifier, Craft, EquipmentSlot, Food, Item, ItemId, ItemSet, Product, Rarity, Tool,
};

pub struct Spyglass;

impl Item for Spyglass {
    const NAME: &'static str = "minecraft:spyglass";
    const ID: ItemId = ItemId(1188);
    const CLASS: &'static str = "SpyglassItem";
    const MAX_STACK_SIZE: u8 = 1;
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
    const CRAFTS: &'static [Craft] = &[Craft::Shaped {
        id: "minecraft:spyglass",
        width: 1,
        height: 3,
        symmetrical: true,
        cells: &[
            Some(ItemSet::Direct(&[ItemId(1016)])),
            Some(ItemSet::Direct(&[ItemId(1020)])),
            Some(ItemSet::Direct(&[ItemId(1020)])),
        ],
        product: Product::of(1),
    }];
    type Behavior = crate::family::Plain;
}
