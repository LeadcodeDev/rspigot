use rspigot_blocks::BlockId;

use crate::{BlockSet, EquipmentSlot, Food, Item, ItemId, ItemSet, Rarity, Tool, ToolRule, tags};

pub struct GoldenHoe;

impl Item for GoldenHoe {
    const NAME: &'static str = "minecraft:golden_hoe";
    const ID: ItemId = ItemId(1044);
    const CLASS: &'static str = "Item";
    const MAX_STACK_SIZE: u8 = 1;
    const MAX_DAMAGE: Option<u32> = Some(32);
    const RARITY: Rarity = Rarity::Common;
    const BLOCK: Option<BlockId> = None;
    const TOOL: Option<Tool> = Some(Tool {
        rules: &[
            ToolRule {
                blocks: BlockSet::Tag(&tags::BLOCK_INCORRECT_FOR_GOLD_TOOL),
                speed: None,
                correct_for_drops: Some(false),
            },
            ToolRule {
                blocks: BlockSet::Tag(&tags::BLOCK_MINEABLE_HOE),
                speed: Some(12.0),
                correct_for_drops: Some(true),
            },
        ],
        default_mining_speed: 1.0,
        damage_per_block: 1,
        can_destroy_blocks_in_creative: true,
    });
    const FOOD: Option<Food> = None;
    const EQUIPPABLE: Option<EquipmentSlot> = None;
    const ENCHANTABLE: Option<u32> = Some(22);
    const REPAIRABLE: Option<ItemSet> = Some(ItemSet::Tag(&tags::ITEM_GOLD_TOOL_MATERIALS));
    const DAMAGE_RESISTANT: Option<&'static str> = None;
    const USE_REMAINDER: Option<ItemId> = None;
    type Behavior = crate::family::Plain;
}
