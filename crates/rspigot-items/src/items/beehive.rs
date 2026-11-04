use rspigot_blocks::BlockId;

use crate::{
    AttributeModifier, Craft, EquipmentSlot, Food, Item, ItemId, ItemSet, Product, Rarity, Tool,
    tags,
};

pub struct Beehive;

impl Item for Beehive {
    const NAME: &'static str = "minecraft:beehive";
    const ID: ItemId = ItemId(1532);
    const CLASS: &'static str = "BlockItem";
    const MAX_STACK_SIZE: u8 = 64;
    const MAX_DAMAGE: Option<u32> = None;
    const WEAPON_DAMAGE: Option<u32> = None;
    const RARITY: Rarity = Rarity::Common;
    const BLOCK: Option<BlockId> = Some(BlockId(1002));
    const TOOL: Option<Tool> = None;
    const FOOD: Option<Food> = None;
    const EQUIPPABLE: Option<EquipmentSlot> = None;
    const ENCHANTABLE: Option<u32> = None;
    const REPAIRABLE: Option<ItemSet> = None;
    const DAMAGE_RESISTANT: Option<&'static str> = None;
    const USE_REMAINDER: Option<ItemId> = None;
    const ATTRIBUTE_MODIFIERS: &'static [AttributeModifier] = &[];
    const CRAFTS: &'static [Craft] = &[Craft::Shaped {
        id: "minecraft:beehive",
        width: 3,
        height: 3,
        symmetrical: true,
        cells: &[
            Some(ItemSet::Tag(&tags::ITEM_PLANKS)),
            Some(ItemSet::Tag(&tags::ITEM_PLANKS)),
            Some(ItemSet::Tag(&tags::ITEM_PLANKS)),
            Some(ItemSet::Direct(&[ItemId(1530)])),
            Some(ItemSet::Direct(&[ItemId(1530)])),
            Some(ItemSet::Direct(&[ItemId(1530)])),
            Some(ItemSet::Tag(&tags::ITEM_PLANKS)),
            Some(ItemSet::Tag(&tags::ITEM_PLANKS)),
            Some(ItemSet::Tag(&tags::ITEM_PLANKS)),
        ],
        product: Product::of(1),
    }];
    type Behavior = crate::family::BlockItem;
}
