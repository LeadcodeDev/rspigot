use rspigot_blocks::BlockId;

use crate::{EquipmentSlot, Food, Item, ItemId, ItemSet, Rarity, Tool};

pub struct FriendPotterySherd;

impl Item for FriendPotterySherd {
    const NAME: &'static str = "minecraft:friend_pottery_sherd";
    const ID: ItemId = ItemId(1607);
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
    type Behavior = crate::family::Plain;
}
