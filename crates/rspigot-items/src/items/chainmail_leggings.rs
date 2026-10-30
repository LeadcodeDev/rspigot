use rspigot_blocks::BlockId;

use crate::{
    AttributeModifier, EquipmentSlot, Food, Item, ItemId, ItemSet, ModifierOperation, Rarity,
    SlotGroup, Tool, tags,
};

pub struct ChainmailLeggings;

impl Item for ChainmailLeggings {
    const NAME: &'static str = "minecraft:chainmail_leggings";
    const ID: ItemId = ItemId(1078);
    const CLASS: &'static str = "Item";
    const MAX_STACK_SIZE: u8 = 1;
    const MAX_DAMAGE: Option<u32> = Some(225);
    const WEAPON_DAMAGE: Option<u32> = None;
    const RARITY: Rarity = Rarity::Uncommon;
    const BLOCK: Option<BlockId> = None;
    const TOOL: Option<Tool> = None;
    const FOOD: Option<Food> = None;
    const EQUIPPABLE: Option<EquipmentSlot> = Some(EquipmentSlot::Legs);
    const ENCHANTABLE: Option<u32> = Some(12);
    const REPAIRABLE: Option<ItemSet> = Some(ItemSet::Tag(&tags::ITEM_REPAIRS_CHAIN_ARMOR));
    const DAMAGE_RESISTANT: Option<&'static str> = None;
    const USE_REMAINDER: Option<ItemId> = None;
    const ATTRIBUTE_MODIFIERS: &'static [AttributeModifier] = &[
        AttributeModifier {
            attribute: "minecraft:armor",
            id: "minecraft:armor.leggings",
            amount: 4.0,
            operation: ModifierOperation::AddValue,
            slot: SlotGroup::Legs,
        },
        AttributeModifier {
            attribute: "minecraft:armor_toughness",
            id: "minecraft:armor.leggings",
            amount: 0.0,
            operation: ModifierOperation::AddValue,
            slot: SlotGroup::Legs,
        },
    ];
    type Behavior = crate::family::Plain;
}
