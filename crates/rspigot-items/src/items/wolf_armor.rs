use rspigot_blocks::BlockId;

use crate::{
    AttributeModifier, EquipmentSlot, Food, Item, ItemId, ItemSet, ModifierOperation, Rarity,
    SlotGroup, Tool, tags,
};

pub struct WolfArmor;

impl Item for WolfArmor {
    const NAME: &'static str = "minecraft:wolf_armor";
    const ID: ItemId = ItemId(1004);
    const CLASS: &'static str = "Item";
    const MAX_STACK_SIZE: u8 = 1;
    const MAX_DAMAGE: Option<u32> = Some(64);
    const WEAPON_DAMAGE: Option<u32> = None;
    const RARITY: Rarity = Rarity::Common;
    const BLOCK: Option<BlockId> = None;
    const TOOL: Option<Tool> = None;
    const FOOD: Option<Food> = None;
    const EQUIPPABLE: Option<EquipmentSlot> = Some(EquipmentSlot::Body);
    const ENCHANTABLE: Option<u32> = None;
    const REPAIRABLE: Option<ItemSet> = Some(ItemSet::Tag(&tags::ITEM_REPAIRS_WOLF_ARMOR));
    const DAMAGE_RESISTANT: Option<&'static str> = None;
    const USE_REMAINDER: Option<ItemId> = None;
    const ATTRIBUTE_MODIFIERS: &'static [AttributeModifier] = &[
        AttributeModifier {
            attribute: "minecraft:armor",
            id: "minecraft:armor.body",
            amount: 11.0,
            operation: ModifierOperation::AddValue,
            slot: SlotGroup::Body,
        },
        AttributeModifier {
            attribute: "minecraft:armor_toughness",
            id: "minecraft:armor.body",
            amount: 0.0,
            operation: ModifierOperation::AddValue,
            slot: SlotGroup::Body,
        },
    ];
    type Behavior = crate::family::Plain;
}
