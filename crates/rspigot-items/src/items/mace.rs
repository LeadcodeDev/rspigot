use rspigot_blocks::BlockId;

use crate::{
    AttributeModifier, EquipmentSlot, Food, Item, ItemId, ItemSet, ModifierOperation, Rarity,
    SlotGroup, Tool,
};

pub struct Mace;

impl Item for Mace {
    const NAME: &'static str = "minecraft:mace";
    const ID: ItemId = ItemId(1374);
    const CLASS: &'static str = "MaceItem";
    const MAX_STACK_SIZE: u8 = 1;
    const MAX_DAMAGE: Option<u32> = Some(500);
    const WEAPON_DAMAGE: Option<u32> = Some(1);
    const RARITY: Rarity = Rarity::Epic;
    const BLOCK: Option<BlockId> = None;
    const TOOL: Option<Tool> = Some(Tool {
        rules: &[],
        default_mining_speed: 1.0,
        damage_per_block: 2,
        can_destroy_blocks_in_creative: false,
    });
    const FOOD: Option<Food> = None;
    const EQUIPPABLE: Option<EquipmentSlot> = None;
    const ENCHANTABLE: Option<u32> = Some(15);
    const REPAIRABLE: Option<ItemSet> = Some(ItemSet::Direct(&[ItemId(1373)]));
    const DAMAGE_RESISTANT: Option<&'static str> = None;
    const USE_REMAINDER: Option<ItemId> = None;
    const ATTRIBUTE_MODIFIERS: &'static [AttributeModifier] = &[
        AttributeModifier {
            attribute: "minecraft:attack_damage",
            id: "minecraft:base_attack_damage",
            amount: 5.0,
            operation: ModifierOperation::AddValue,
            slot: SlotGroup::Mainhand,
        },
        AttributeModifier {
            attribute: "minecraft:attack_speed",
            id: "minecraft:base_attack_speed",
            amount: -3.4000000953674316,
            operation: ModifierOperation::AddValue,
            slot: SlotGroup::Mainhand,
        },
    ];
    type Behavior = crate::family::Plain;
}
