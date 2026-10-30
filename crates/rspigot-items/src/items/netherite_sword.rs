use rspigot_blocks::BlockId;

use crate::{
    AttributeModifier, BlockSet, EquipmentSlot, Food, Item, ItemId, ItemSet, ModifierOperation,
    Rarity, SlotGroup, Tool, ToolRule, tags,
};

pub struct NetheriteSword;

impl Item for NetheriteSword {
    const NAME: &'static str = "minecraft:netherite_sword";
    const ID: ItemId = ItemId(1055);
    const CLASS: &'static str = "Item";
    const MAX_STACK_SIZE: u8 = 1;
    const MAX_DAMAGE: Option<u32> = Some(2031);
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
    const REPAIRABLE: Option<ItemSet> = Some(ItemSet::Tag(&tags::ITEM_NETHERITE_TOOL_MATERIALS));
    const DAMAGE_RESISTANT: Option<&'static str> = Some("minecraft:is_fire");
    const USE_REMAINDER: Option<ItemId> = None;
    const ATTRIBUTE_MODIFIERS: &'static [AttributeModifier] = &[
        AttributeModifier {
            attribute: "minecraft:attack_damage",
            id: "minecraft:base_attack_damage",
            amount: 7.0,
            operation: ModifierOperation::AddValue,
            slot: SlotGroup::Mainhand,
        },
        AttributeModifier {
            attribute: "minecraft:attack_speed",
            id: "minecraft:base_attack_speed",
            amount: -2.4000000953674316,
            operation: ModifierOperation::AddValue,
            slot: SlotGroup::Mainhand,
        },
    ];
    type Behavior = crate::family::Plain;
}
