use rspigot_blocks::BlockId;

use crate::{
    AttributeModifier, Craft, EquipmentSlot, Food, Item, ItemId, ItemSet, Product, Rarity, Tool,
};

pub struct Compass;

impl Item for Compass {
    const NAME: &'static str = "minecraft:compass";
    const ID: ItemId = ItemId(1151);
    const CLASS: &'static str = "CompassItem";
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
        id: "minecraft:compass",
        width: 3,
        height: 3,
        symmetrical: true,
        cells: &[
            None,
            Some(ItemSet::Direct(&[ItemId(1018)])),
            None,
            Some(ItemSet::Direct(&[ItemId(1018)])),
            Some(ItemSet::Direct(&[ItemId(824)])),
            Some(ItemSet::Direct(&[ItemId(1018)])),
            None,
            Some(ItemSet::Direct(&[ItemId(1018)])),
            None,
        ],
        product: Product::of(1),
    }];
    type Behavior = crate::family::Plain;
}
