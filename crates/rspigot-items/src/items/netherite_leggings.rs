use rspigot_blocks::BlockId;

use crate::{
    AttributeModifier, Craft, EquipmentSlot, Food, Item, ItemId, ItemSet, ModifierOperation, Rarity,
    SlotGroup, Tool, tags,
};

pub struct NetheriteLeggings;

impl Item for NetheriteLeggings {
    const NAME: &'static str = "minecraft:netherite_leggings";
    const ID: ItemId = ItemId(1094);
    const CLASS: &'static str = "Item";
    const MAX_STACK_SIZE: u8 = 1;
    const MAX_DAMAGE: Option<u32> = Some(555);
    const WEAPON_DAMAGE: Option<u32> = None;
    const RARITY: Rarity = Rarity::Common;
    const BLOCK: Option<BlockId> = None;
    const TOOL: Option<Tool> = None;
    const FOOD: Option<Food> = None;
    const EQUIPPABLE: Option<EquipmentSlot> = Some(EquipmentSlot::Legs);
    const ENCHANTABLE: Option<u32> = Some(15);
    const REPAIRABLE: Option<ItemSet> = Some(ItemSet::Tag(&tags::ITEM_REPAIRS_NETHERITE_ARMOR));
    const DAMAGE_RESISTANT: Option<&'static str> = Some("minecraft:is_fire");
    const USE_REMAINDER: Option<ItemId> = None;
    const ATTRIBUTE_MODIFIERS: &'static [AttributeModifier] = &[
        AttributeModifier {
            attribute: "minecraft:armor",
            id: "minecraft:armor.leggings",
            amount: 6.0,
            operation: ModifierOperation::AddValue,
            slot: SlotGroup::Legs,
        },
        AttributeModifier {
            attribute: "minecraft:armor_toughness",
            id: "minecraft:armor.leggings",
            amount: 3.0,
            operation: ModifierOperation::AddValue,
            slot: SlotGroup::Legs,
        },
        AttributeModifier {
            attribute: "minecraft:knockback_resistance",
            id: "minecraft:armor.leggings",
            amount: 0.10000000149011612,
            operation: ModifierOperation::AddValue,
            slot: SlotGroup::Legs,
        },
    ];
    const CRAFTS: &'static [Craft] = &[
        Craft::SmithingTransform {
            template: ItemSet::Direct(&[ItemId(1579)]),
            base: ItemSet::Direct(&[ItemId(1086)]),
            addition: ItemSet::Tag(&tags::ITEM_NETHERITE_TOOL_MATERIALS),
        },
        Craft::RepairItem,
    ];
    type Behavior = crate::family::Plain;
}
