use rspigot_blocks::BlockId;

use crate::{
    AttributeModifier, EquipmentSlot, Food, Item, ItemId, ItemSet, ModifierOperation, Rarity,
    SlotGroup, Tool, tags,
};

pub struct DiamondSpear;

impl Item for DiamondSpear {
    const NAME: &'static str = "minecraft:diamond_spear";
    const ID: ItemId = ItemId(1452);
    const CLASS: &'static str = "Item";
    const MAX_STACK_SIZE: u8 = 1;
    const MAX_DAMAGE: Option<u32> = Some(1561);
    const WEAPON_DAMAGE: Option<u32> = Some(1);
    const RARITY: Rarity = Rarity::Common;
    const BLOCK: Option<BlockId> = None;
    const TOOL: Option<Tool> = None;
    const FOOD: Option<Food> = None;
    const EQUIPPABLE: Option<EquipmentSlot> = None;
    const ENCHANTABLE: Option<u32> = Some(10);
    const REPAIRABLE: Option<ItemSet> = Some(ItemSet::Tag(&tags::ITEM_DIAMOND_TOOL_MATERIALS));
    const DAMAGE_RESISTANT: Option<&'static str> = None;
    const USE_REMAINDER: Option<ItemId> = None;
    const ATTRIBUTE_MODIFIERS: &'static [AttributeModifier] = &[
        AttributeModifier {
            attribute: "minecraft:attack_damage",
            id: "minecraft:base_attack_damage",
            amount: 3.0,
            operation: ModifierOperation::AddValue,
            slot: SlotGroup::Mainhand,
        },
        AttributeModifier {
            attribute: "minecraft:attack_speed",
            id: "minecraft:base_attack_speed",
            amount: -3.0476189851760864,
            operation: ModifierOperation::AddValue,
            slot: SlotGroup::Mainhand,
        },
    ];
    type Behavior = crate::family::Plain;
}
