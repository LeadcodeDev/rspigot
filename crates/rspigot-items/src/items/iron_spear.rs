use rspigot_blocks::BlockId;

use crate::{EquipmentSlot, Food, Item, ItemId, ItemSet, Rarity, Tool, tags};

pub struct IronSpear;

impl Item for IronSpear {
    const NAME: &'static str = "minecraft:iron_spear";
    const ID: ItemId = ItemId(1450);
    const CLASS: &'static str = "Item";
    const MAX_STACK_SIZE: u8 = 1;
    const MAX_DAMAGE: Option<u32> = Some(250);
    const RARITY: Rarity = Rarity::Common;
    const BLOCK: Option<BlockId> = None;
    const TOOL: Option<Tool> = None;
    const FOOD: Option<Food> = None;
    const EQUIPPABLE: Option<EquipmentSlot> = None;
    const ENCHANTABLE: Option<u32> = Some(14);
    const REPAIRABLE: Option<ItemSet> = Some(ItemSet::Tag(&tags::ITEM_IRON_TOOL_MATERIALS));
    const DAMAGE_RESISTANT: Option<&'static str> = None;
    const USE_REMAINDER: Option<ItemId> = None;
    type Behavior = crate::family::Plain;
}
