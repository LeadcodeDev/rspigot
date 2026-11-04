use rspigot_blocks::BlockId;

use crate::{
    AttributeModifier, Craft, EquipmentSlot, Food, Item, ItemId, ItemSet, ModifierOperation, Rarity,
    SlotGroup, Tool, tags,
};

pub struct NetheriteHelmet;

impl Item for NetheriteHelmet {
    const NAME: &'static str = "minecraft:netherite_helmet";
    const ID: ItemId = ItemId(1092);
    const CLASS: &'static str = "Item";
    const MAX_STACK_SIZE: u8 = 1;
    const MAX_DAMAGE: Option<u32> = Some(407);
    const WEAPON_DAMAGE: Option<u32> = None;
    const RARITY: Rarity = Rarity::Common;
    const BLOCK: Option<BlockId> = None;
    const TOOL: Option<Tool> = None;
    const FOOD: Option<Food> = None;
    const EQUIPPABLE: Option<EquipmentSlot> = Some(EquipmentSlot::Head);
    const ENCHANTABLE: Option<u32> = Some(15);
    const REPAIRABLE: Option<ItemSet> = Some(ItemSet::Tag(&tags::ITEM_REPAIRS_NETHERITE_ARMOR));
    const DAMAGE_RESISTANT: Option<&'static str> = Some("minecraft:is_fire");
    const USE_REMAINDER: Option<ItemId> = None;
    const ATTRIBUTE_MODIFIERS: &'static [AttributeModifier] = &[
        AttributeModifier {
            attribute: "minecraft:armor",
            id: "minecraft:armor.helmet",
            amount: 3.0,
            operation: ModifierOperation::AddValue,
            slot: SlotGroup::Head,
        },
        AttributeModifier {
            attribute: "minecraft:armor_toughness",
            id: "minecraft:armor.helmet",
            amount: 3.0,
            operation: ModifierOperation::AddValue,
            slot: SlotGroup::Head,
        },
        AttributeModifier {
            attribute: "minecraft:knockback_resistance",
            id: "minecraft:armor.helmet",
            amount: 0.10000000149011612,
            operation: ModifierOperation::AddValue,
            slot: SlotGroup::Head,
        },
    ];
    const CRAFTS: &'static [Craft] = &[
        Craft::SmithingTransform {
            id: "minecraft:netherite_helmet_smithing",
            template: ItemSet::Direct(&[ItemId(1579)]),
            base: ItemSet::Direct(&[ItemId(1084)]),
            addition: ItemSet::Tag(&tags::ITEM_NETHERITE_TOOL_MATERIALS),
        },
        Craft::RepairItem { id: "minecraft:repair_item" },
    ];
    type Behavior = crate::family::Plain;
}
