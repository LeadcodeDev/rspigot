use rspigot_blocks::BlockId;

use crate::{
    AttributeModifier, EquipmentSlot, Food, Item, ItemId, ItemSet, ModifierOperation, Rarity,
    SlotGroup, Tool,
};

pub struct DragonHead;

impl Item for DragonHead {
    const NAME: &'static str = "minecraft:dragon_head";
    const ID: ItemId = ItemId(1389);
    const CLASS: &'static str = "StandingAndWallBlockItem";
    const MAX_STACK_SIZE: u8 = 64;
    const MAX_DAMAGE: Option<u32> = None;
    const WEAPON_DAMAGE: Option<u32> = None;
    const RARITY: Rarity = Rarity::Epic;
    const BLOCK: Option<BlockId> = Some(BlockId(516));
    const TOOL: Option<Tool> = None;
    const FOOD: Option<Food> = None;
    const EQUIPPABLE: Option<EquipmentSlot> = Some(EquipmentSlot::Head);
    const ENCHANTABLE: Option<u32> = None;
    const REPAIRABLE: Option<ItemSet> = None;
    const DAMAGE_RESISTANT: Option<&'static str> = None;
    const USE_REMAINDER: Option<ItemId> = None;
    const ATTRIBUTE_MODIFIERS: &'static [AttributeModifier] = &[
        AttributeModifier {
            attribute: "minecraft:waypoint_transmit_range",
            id: "minecraft:waypoint_transmit_range_hide",
            amount: -1.0,
            operation: ModifierOperation::AddMultipliedTotal,
            slot: SlotGroup::Head,
        },
    ];
    type Behavior = crate::family::BlockItem;
}
