use rspigot_blocks::BlockId;

use crate::{AttributeModifier, EquipmentSlot, Food, Item, ItemId, ItemSet, Rarity, Tool};

pub struct SheafPotterySherd;

impl Item for SheafPotterySherd {
    const NAME: &'static str = "minecraft:sheaf_pottery_sherd";
    const ID: ItemId = ItemId(1617);
    const CLASS: &'static str = "Item";
    const MAX_STACK_SIZE: u8 = 64;
    const MAX_DAMAGE: Option<u32> = None;
    const RARITY: Rarity = Rarity::Uncommon;
    const BLOCK: Option<BlockId> = None;
    const TOOL: Option<Tool> = None;
    const FOOD: Option<Food> = None;
    const EQUIPPABLE: Option<EquipmentSlot> = None;
    const ENCHANTABLE: Option<u32> = None;
    const REPAIRABLE: Option<ItemSet> = None;
    const DAMAGE_RESISTANT: Option<&'static str> = None;
    const USE_REMAINDER: Option<ItemId> = None;
    const ATTRIBUTE_MODIFIERS: &'static [AttributeModifier] = &[];
    type Behavior = crate::family::Plain;
}
