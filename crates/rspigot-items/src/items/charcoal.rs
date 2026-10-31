use rspigot_blocks::BlockId;

use crate::{
    AttributeModifier, CookingKind, Craft, EquipmentSlot, Food, Item, ItemId, ItemSet, Rarity, Tool,
    tags,
};

pub struct Charcoal;

impl Item for Charcoal {
    const NAME: &'static str = "minecraft:charcoal";
    const ID: ItemId = ItemId(1011);
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
    const CRAFTS: &'static [Craft] = &[Craft::Cooking {
        kind: CookingKind::Smelting,
        ingredient: ItemSet::Tag(&tags::ITEM_LOGS_THAT_BURN),
        experience: 0.15,
        cooking_time: 200,
    }];
    type Behavior = crate::family::Plain;
}
