use rspigot_blocks::BlockId;

use crate::{BlockSet, EquipmentSlot, Food, Item, ItemId, ItemSet, Rarity, Tool, ToolRule, tags};

pub struct Shears;

impl Item for Shears {
    const NAME: &'static str = "minecraft:shears";
    const ID: ItemId = ItemId(1255);
    const CLASS: &'static str = "ShearsItem";
    const MAX_STACK_SIZE: u8 = 1;
    const MAX_DAMAGE: Option<u32> = Some(238);
    const RARITY: Rarity = Rarity::Common;
    const BLOCK: Option<BlockId> = None;
    const TOOL: Option<Tool> = Some(Tool {
        rules: &[
            ToolRule {
                blocks: BlockSet::Direct(&[BlockId(139)]),
                speed: Some(15.0),
                correct_for_drops: Some(true),
            },
            ToolRule {
                blocks: BlockSet::Tag(&tags::BLOCK_SHEARS_EXTREME_BREAKING_SPEED),
                speed: Some(15.0),
                correct_for_drops: None,
            },
            ToolRule {
                blocks: BlockSet::Tag(&tags::BLOCK_SHEARS_MAJOR_BREAKING_SPEED),
                speed: Some(5.0),
                correct_for_drops: None,
            },
            ToolRule {
                blocks: BlockSet::Tag(&tags::BLOCK_SHEARS_MINOR_BREAKING_SPEED),
                speed: Some(2.0),
                correct_for_drops: None,
            },
        ],
        default_mining_speed: 1.0,
        damage_per_block: 1,
        can_destroy_blocks_in_creative: true,
    });
    const FOOD: Option<Food> = None;
    const EQUIPPABLE: Option<EquipmentSlot> = None;
    const ENCHANTABLE: Option<u32> = None;
    const REPAIRABLE: Option<ItemSet> = None;
    const DAMAGE_RESISTANT: Option<&'static str> = None;
    const USE_REMAINDER: Option<ItemId> = None;
    type Behavior = crate::family::Plain;
}
