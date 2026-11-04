use rspigot_blocks::BlockId;

use crate::{
    AttributeModifier, Craft, EquipmentSlot, Food, Item, ItemId, ItemSet, Product, Rarity, Tool,
};

pub struct Map;

impl Item for Map {
    const NAME: &'static str = "minecraft:map";
    const ID: ItemId = ItemId(1382);
    const CLASS: &'static str = "EmptyMapItem";
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
    const CRAFTS: &'static [Craft] = &[Craft::Shaped {
        id: "minecraft:map",
        width: 3,
        height: 3,
        symmetrical: true,
        cells: &[
            Some(ItemSet::Direct(&[ItemId(1145)])),
            Some(ItemSet::Direct(&[ItemId(1145)])),
            Some(ItemSet::Direct(&[ItemId(1145)])),
            Some(ItemSet::Direct(&[ItemId(1145)])),
            Some(ItemSet::Direct(&[ItemId(1151)])),
            Some(ItemSet::Direct(&[ItemId(1145)])),
            Some(ItemSet::Direct(&[ItemId(1145)])),
            Some(ItemSet::Direct(&[ItemId(1145)])),
            Some(ItemSet::Direct(&[ItemId(1145)])),
        ],
        product: Product::of(1),
    }];
    type Behavior = crate::family::Plain;
}
