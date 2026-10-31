use rspigot_blocks::BlockId;

use crate::{
    AttributeModifier, BlockSet, Craft, EquipmentSlot, Food, Item, ItemId, ItemSet,
    ModifierOperation, Product, Rarity, SlotGroup, Tool, ToolRule, tags,
};

pub struct WoodenHoe;

impl Item for WoodenHoe {
    const NAME: &'static str = "minecraft:wooden_hoe";
    const ID: ItemId = ItemId(1029);
    const CLASS: &'static str = "Item";
    const MAX_STACK_SIZE: u8 = 1;
    const MAX_DAMAGE: Option<u32> = Some(59);
    const WEAPON_DAMAGE: Option<u32> = Some(2);
    const RARITY: Rarity = Rarity::Common;
    const BLOCK: Option<BlockId> = None;
    const TOOL: Option<Tool> = Some(Tool {
        rules: &[
            ToolRule {
                blocks: BlockSet::Tag(&tags::BLOCK_INCORRECT_FOR_WOODEN_TOOL),
                speed: None,
                correct_for_drops: Some(false),
            },
            ToolRule {
                blocks: BlockSet::Tag(&tags::BLOCK_MINEABLE_HOE),
                speed: Some(2.0),
                correct_for_drops: Some(true),
            },
        ],
        default_mining_speed: 1.0,
        damage_per_block: 1,
        can_destroy_blocks_in_creative: true,
    });
    const FOOD: Option<Food> = None;
    const EQUIPPABLE: Option<EquipmentSlot> = None;
    const ENCHANTABLE: Option<u32> = Some(15);
    const REPAIRABLE: Option<ItemSet> = Some(ItemSet::Tag(&tags::ITEM_WOODEN_TOOL_MATERIALS));
    const DAMAGE_RESISTANT: Option<&'static str> = None;
    const USE_REMAINDER: Option<ItemId> = None;
    const ATTRIBUTE_MODIFIERS: &'static [AttributeModifier] = &[
        AttributeModifier {
            attribute: "minecraft:attack_damage",
            id: "minecraft:base_attack_damage",
            amount: 0.0,
            operation: ModifierOperation::AddValue,
            slot: SlotGroup::Mainhand,
        },
        AttributeModifier {
            attribute: "minecraft:attack_speed",
            id: "minecraft:base_attack_speed",
            amount: -3.0,
            operation: ModifierOperation::AddValue,
            slot: SlotGroup::Mainhand,
        },
    ];
    const CRAFTS: &'static [Craft] = &[
        Craft::RepairItem,
        Craft::Shaped {
            width: 2,
            height: 3,
            symmetrical: false,
            cells: &[
                Some(ItemSet::Tag(&tags::ITEM_WOODEN_TOOL_MATERIALS)),
                Some(ItemSet::Tag(&tags::ITEM_WOODEN_TOOL_MATERIALS)),
                None,
                Some(ItemSet::Direct(&[ItemId(1060)])),
                None,
                Some(ItemSet::Direct(&[ItemId(1060)])),
            ],
            product: Product::of(1),
        },
    ];
    type Behavior = crate::family::Plain;
}
