use rspigot_blocks::BlockId;

use crate::{
    AttributeModifier, Craft, EquipmentSlot, Food, Item, ItemId, ItemSet, Product, Rarity, Tool,
    tags,
};

pub struct Cake;

impl Item for Cake {
    const NAME: &'static str = "minecraft:cake";
    const ID: ItemId = ItemId(1218);
    const CLASS: &'static str = "BlockItem";
    const MAX_STACK_SIZE: u8 = 1;
    const MAX_DAMAGE: Option<u32> = None;
    const WEAPON_DAMAGE: Option<u32> = None;
    const RARITY: Rarity = Rarity::Common;
    const BLOCK: Option<BlockId> = Some(BlockId(347));
    const TOOL: Option<Tool> = None;
    const FOOD: Option<Food> = None;
    const EQUIPPABLE: Option<EquipmentSlot> = None;
    const ENCHANTABLE: Option<u32> = None;
    const REPAIRABLE: Option<ItemSet> = None;
    const DAMAGE_RESISTANT: Option<&'static str> = None;
    const USE_REMAINDER: Option<ItemId> = None;
    const ATTRIBUTE_MODIFIERS: &'static [AttributeModifier] = &[];
    const CRAFTS: &'static [Craft] = &[Craft::Shaped {
        id: "minecraft:cake",
        width: 3,
        height: 3,
        symmetrical: true,
        cells: &[
            Some(ItemSet::Direct(&[ItemId(1134)])),
            Some(ItemSet::Direct(&[ItemId(1134)])),
            Some(ItemSet::Direct(&[ItemId(1134)])),
            Some(ItemSet::Direct(&[ItemId(1217)])),
            Some(ItemSet::Tag(&tags::ITEM_EGGS)),
            Some(ItemSet::Direct(&[ItemId(1217)])),
            Some(ItemSet::Direct(&[ItemId(1066)])),
            Some(ItemSet::Direct(&[ItemId(1066)])),
            Some(ItemSet::Direct(&[ItemId(1066)])),
        ],
        product: Product::of(1),
    }];
    type Behavior = crate::family::BlockItem;
}
