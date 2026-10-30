use rspigot_blocks::BlockId;

use crate::{AttributeModifier, EquipmentSlot, Food, Item, ItemId, ItemSet, Rarity, Tool};

pub struct WeatheredLightningRod;

impl Item for WeatheredLightningRod {
    const NAME: &'static str = "minecraft:weathered_lightning_rod";
    const ID: ItemId = ItemId(842);
    const CLASS: &'static str = "BlockItem";
    const MAX_STACK_SIZE: u8 = 64;
    const MAX_DAMAGE: Option<u32> = None;
    const RARITY: Rarity = Rarity::Common;
    const BLOCK: Option<BlockId> = Some(BlockId(1216));
    const TOOL: Option<Tool> = None;
    const FOOD: Option<Food> = None;
    const EQUIPPABLE: Option<EquipmentSlot> = None;
    const ENCHANTABLE: Option<u32> = None;
    const REPAIRABLE: Option<ItemSet> = None;
    const DAMAGE_RESISTANT: Option<&'static str> = None;
    const USE_REMAINDER: Option<ItemId> = None;
    const ATTRIBUTE_MODIFIERS: &'static [AttributeModifier] = &[];
    type Behavior = crate::family::BlockItem;
}
