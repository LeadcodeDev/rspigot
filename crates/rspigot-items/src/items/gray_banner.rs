use rspigot_blocks::BlockId;

use crate::{AttributeModifier, EquipmentSlot, Food, Item, ItemId, ItemSet, Rarity, Tool};

pub struct GrayBanner;

impl Item for GrayBanner {
    const NAME: &'static str = "minecraft:gray_banner";
    const ID: ItemId = ItemId(1424);
    const CLASS: &'static str = "BannerItem";
    const MAX_STACK_SIZE: u8 = 16;
    const MAX_DAMAGE: Option<u32> = None;
    const RARITY: Rarity = Rarity::Common;
    const BLOCK: Option<BlockId> = Some(BlockId(624));
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
