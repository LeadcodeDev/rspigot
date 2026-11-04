use rspigot_blocks::BlockId;

use crate::{
    AttributeModifier, Craft, EquipmentSlot, Food, Item, ItemId, ItemSet, Rarity, Tool, tags,
};

pub struct WrittenBook;

impl Item for WrittenBook {
    const NAME: &'static str = "minecraft:written_book";
    const ID: ItemId = ItemId(1372);
    const CLASS: &'static str = "WrittenBookItem";
    const MAX_STACK_SIZE: u8 = 16;
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
    const CRAFTS: &'static [Craft] = &[Craft::BookCloning {
        id: "minecraft:book_cloning",
        source: ItemSet::Direct(&[ItemId(1372)]),
        material: ItemSet::Tag(&tags::ITEM_BOOK_CLONING_TARGET),
    }];
    type Behavior = crate::family::Plain;
}
