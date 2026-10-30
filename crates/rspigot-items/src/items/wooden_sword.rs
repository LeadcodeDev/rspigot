use rspigot_blocks::BlockId;

use crate::{BlockSet, EquipmentSlot, Food, Item, ItemId, ItemSet, Rarity, Tool, ToolRule, tags};

pub struct WoodenSword;

impl Item for WoodenSword {
    const NAME: &'static str = "minecraft:wooden_sword";
    const ID: ItemId = ItemId(1025);
    const CLASS: &'static str = "Item";
    const MAX_STACK_SIZE: u8 = 1;
    const MAX_DAMAGE: Option<u32> = Some(59);
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
                blocks: BlockSet::Tag(&tags::BLOCK_SWORD_INSTANTLY_MINES),
                speed: Some(3.4028235e38),
                correct_for_drops: None,
            },
            ToolRule {
                blocks: BlockSet::Tag(&tags::BLOCK_SWORD_EFFICIENT),
                speed: Some(1.5),
                correct_for_drops: None,
            },
        ],
        default_mining_speed: 1.0,
        damage_per_block: 2,
        can_destroy_blocks_in_creative: false,
    });
    const FOOD: Option<Food> = None;
    const EQUIPPABLE: Option<EquipmentSlot> = None;
    const ENCHANTABLE: Option<u32> = Some(15);
    const REPAIRABLE: Option<ItemSet> = Some(ItemSet::Tag(&tags::ITEM_WOODEN_TOOL_MATERIALS));
    const DAMAGE_RESISTANT: Option<&'static str> = None;
    const USE_REMAINDER: Option<ItemId> = None;
    type Behavior = crate::family::Plain;
}
