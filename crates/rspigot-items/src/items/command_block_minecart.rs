use rspigot_blocks::BlockId;

use crate::{EquipmentSlot, Food, Item, ItemId, ItemSet, Rarity, Tool};

pub struct CommandBlockMinecart;

impl Item for CommandBlockMinecart {
    const NAME: &'static str = "minecraft:command_block_minecart";
    const ID: ItemId = ItemId(1414);
    const CLASS: &'static str = "MinecartItem";
    const MAX_STACK_SIZE: u8 = 1;
    const MAX_DAMAGE: Option<u32> = None;
    const RARITY: Rarity = Rarity::Epic;
    const BLOCK: Option<BlockId> = None;
    const TOOL: Option<Tool> = None;
    const FOOD: Option<Food> = None;
    const EQUIPPABLE: Option<EquipmentSlot> = None;
    const ENCHANTABLE: Option<u32> = None;
    const REPAIRABLE: Option<ItemSet> = None;
    const DAMAGE_RESISTANT: Option<&'static str> = None;
    const USE_REMAINDER: Option<ItemId> = None;
    type Behavior = crate::family::Plain;
}
