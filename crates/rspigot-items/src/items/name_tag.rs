use rspigot_blocks::BlockId;

use crate::{
    AttributeModifier, Craft, EquipmentSlot, Food, Item, ItemId, ItemSet, Product, Rarity, Tool,
    tags,
};

pub struct NameTag;

impl Item for NameTag {
    const NAME: &'static str = "minecraft:name_tag";
    const ID: ItemId = ItemId(1413);
    const CLASS: &'static str = "NameTagItem";
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
        id: "minecraft:name_tag",
        width: 2,
        height: 2,
        symmetrical: false,
        cells: &[
            None,
            Some(ItemSet::Tag(&tags::ITEM_METAL_NUGGETS)),
            Some(ItemSet::Direct(&[ItemId(1145)])),
            None,
        ],
        product: Product::of(1),
    }];
    type Behavior = crate::family::Plain;
}
