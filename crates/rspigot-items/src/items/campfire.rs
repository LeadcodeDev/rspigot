use rspigot_blocks::BlockId;

use crate::{
    AttributeModifier, Craft, EquipmentSlot, Food, Item, ItemId, ItemSet, Product, Rarity, Tool,
    tags,
};

pub struct Campfire;

impl Item for Campfire {
    const NAME: &'static str = "minecraft:campfire";
    const ID: ItemId = ItemId(1527);
    const CLASS: &'static str = "BlockItem";
    const MAX_STACK_SIZE: u8 = 64;
    const MAX_DAMAGE: Option<u32> = None;
    const WEAPON_DAMAGE: Option<u32> = None;
    const RARITY: Rarity = Rarity::Common;
    const BLOCK: Option<BlockId> = Some(BlockId(949));
    const TOOL: Option<Tool> = None;
    const FOOD: Option<Food> = None;
    const EQUIPPABLE: Option<EquipmentSlot> = None;
    const ENCHANTABLE: Option<u32> = None;
    const REPAIRABLE: Option<ItemSet> = None;
    const DAMAGE_RESISTANT: Option<&'static str> = None;
    const USE_REMAINDER: Option<ItemId> = None;
    const ATTRIBUTE_MODIFIERS: &'static [AttributeModifier] = &[];
    const CRAFTS: &'static [Craft] = &[Craft::Shaped {
        width: 3,
        height: 3,
        symmetrical: true,
        cells: &[
            None,
            Some(ItemSet::Direct(&[ItemId(1060)])),
            None,
            Some(ItemSet::Direct(&[ItemId(1060)])),
            Some(ItemSet::Tag(&tags::ITEM_COALS)),
            Some(ItemSet::Direct(&[ItemId(1060)])),
            Some(ItemSet::Tag(&tags::ITEM_LOGS)),
            Some(ItemSet::Tag(&tags::ITEM_LOGS)),
            Some(ItemSet::Tag(&tags::ITEM_LOGS)),
        ],
        product: Product::of(1),
    }];
    type Behavior = crate::family::BlockItem;
}
