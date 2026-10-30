use rspigot_blocks::BlockId;

use crate::{AttributeModifier, EquipmentSlot, Food, Item, ItemId, ItemSet, Rarity, Tool};

pub struct FlintAndSteel;

impl Item for FlintAndSteel {
    const NAME: &'static str = "minecraft:flint_and_steel";
    const ID: ItemId = ItemId(1005);
    const CLASS: &'static str = "FlintAndSteelItem";
    const MAX_STACK_SIZE: u8 = 1;
    const MAX_DAMAGE: Option<u32> = Some(64);
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
    type Behavior = crate::family::Plain;
}
