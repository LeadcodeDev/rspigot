use rspigot_blocks::BlockId;

use crate::{
    AttributeModifier, EquipmentSlot, Food, Item, ItemId, ItemSet, ModifierOperation, Rarity,
    SlotGroup, Tool,
};

pub struct NetheriteNautilusArmor;

impl Item for NetheriteNautilusArmor {
    const NAME: &'static str = "minecraft:netherite_nautilus_armor";
    const ID: ItemId = ItemId(1488);
    const CLASS: &'static str = "Item";
    const MAX_STACK_SIZE: u8 = 1;
    const MAX_DAMAGE: Option<u32> = None;
    const RARITY: Rarity = Rarity::Common;
    const BLOCK: Option<BlockId> = None;
    const TOOL: Option<Tool> = None;
    const FOOD: Option<Food> = None;
    const EQUIPPABLE: Option<EquipmentSlot> = Some(EquipmentSlot::Body);
    const ENCHANTABLE: Option<u32> = None;
    const REPAIRABLE: Option<ItemSet> = None;
    const DAMAGE_RESISTANT: Option<&'static str> = Some("minecraft:is_fire");
    const USE_REMAINDER: Option<ItemId> = None;
    const ATTRIBUTE_MODIFIERS: &'static [AttributeModifier] = &[
        AttributeModifier {
            attribute: "minecraft:armor",
            id: "minecraft:armor.body",
            amount: 19.0,
            operation: ModifierOperation::AddValue,
            slot: SlotGroup::Body,
        },
        AttributeModifier {
            attribute: "minecraft:armor_toughness",
            id: "minecraft:armor.body",
            amount: 3.0,
            operation: ModifierOperation::AddValue,
            slot: SlotGroup::Body,
        },
        AttributeModifier {
            attribute: "minecraft:knockback_resistance",
            id: "minecraft:armor.body",
            amount: 0.10000000149011612,
            operation: ModifierOperation::AddValue,
            slot: SlotGroup::Body,
        },
    ];
    type Behavior = crate::family::Plain;
}
