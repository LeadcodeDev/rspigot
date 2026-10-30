use rspigot_blocks::BlockId;

use crate::{
    AttributeModifier, EquipmentSlot, Food, Item, ItemId, ItemSet, ModifierOperation, Rarity,
    SlotGroup, Tool, tags,
};

pub struct ChainmailChestplate;

impl Item for ChainmailChestplate {
    const NAME: &'static str = "minecraft:chainmail_chestplate";
    const ID: ItemId = ItemId(1077);
    const CLASS: &'static str = "Item";
    const MAX_STACK_SIZE: u8 = 1;
    const MAX_DAMAGE: Option<u32> = Some(240);
    const WEAPON_DAMAGE: Option<u32> = None;
    const RARITY: Rarity = Rarity::Uncommon;
    const BLOCK: Option<BlockId> = None;
    const TOOL: Option<Tool> = None;
    const FOOD: Option<Food> = None;
    const EQUIPPABLE: Option<EquipmentSlot> = Some(EquipmentSlot::Chest);
    const ENCHANTABLE: Option<u32> = Some(12);
    const REPAIRABLE: Option<ItemSet> = Some(ItemSet::Tag(&tags::ITEM_REPAIRS_CHAIN_ARMOR));
    const DAMAGE_RESISTANT: Option<&'static str> = None;
    const USE_REMAINDER: Option<ItemId> = None;
    const ATTRIBUTE_MODIFIERS: &'static [AttributeModifier] = &[
        AttributeModifier {
            attribute: "minecraft:armor",
            id: "minecraft:armor.chestplate",
            amount: 5.0,
            operation: ModifierOperation::AddValue,
            slot: SlotGroup::Chest,
        },
        AttributeModifier {
            attribute: "minecraft:armor_toughness",
            id: "minecraft:armor.chestplate",
            amount: 0.0,
            operation: ModifierOperation::AddValue,
            slot: SlotGroup::Chest,
        },
    ];
    type Behavior = crate::family::Plain;
}
