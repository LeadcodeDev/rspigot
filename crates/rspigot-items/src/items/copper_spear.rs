use rspigot_blocks::BlockId;

use crate::{
    AttributeModifier, Craft, EquipmentSlot, Food, Item, ItemId, ItemSet, ModifierOperation,
    Product, Rarity, SlotGroup, Tool, tags,
};

pub struct CopperSpear;

impl Item for CopperSpear {
    const NAME: &'static str = "minecraft:copper_spear";
    const ID: ItemId = ItemId(1449);
    const CLASS: &'static str = "Item";
    const MAX_STACK_SIZE: u8 = 1;
    const MAX_DAMAGE: Option<u32> = Some(190);
    const WEAPON_DAMAGE: Option<u32> = Some(1);
    const RARITY: Rarity = Rarity::Common;
    const BLOCK: Option<BlockId> = None;
    const TOOL: Option<Tool> = None;
    const FOOD: Option<Food> = None;
    const EQUIPPABLE: Option<EquipmentSlot> = None;
    const ENCHANTABLE: Option<u32> = Some(13);
    const REPAIRABLE: Option<ItemSet> = Some(ItemSet::Tag(&tags::ITEM_COPPER_TOOL_MATERIALS));
    const DAMAGE_RESISTANT: Option<&'static str> = None;
    const USE_REMAINDER: Option<ItemId> = None;
    const ATTRIBUTE_MODIFIERS: &'static [AttributeModifier] = &[
        AttributeModifier {
            attribute: "minecraft:attack_damage",
            id: "minecraft:base_attack_damage",
            amount: 1.0,
            operation: ModifierOperation::AddValue,
            slot: SlotGroup::Mainhand,
        },
        AttributeModifier {
            attribute: "minecraft:attack_speed",
            id: "minecraft:base_attack_speed",
            amount: -2.8235294818878174,
            operation: ModifierOperation::AddValue,
            slot: SlotGroup::Mainhand,
        },
    ];
    const CRAFTS: &'static [Craft] = &[Craft::Shaped {
        width: 3,
        height: 3,
        symmetrical: false,
        cells: &[
            None,
            None,
            Some(ItemSet::Tag(&tags::ITEM_COPPER_TOOL_MATERIALS)),
            None,
            Some(ItemSet::Direct(&[ItemId(1060)])),
            None,
            Some(ItemSet::Direct(&[ItemId(1060)])),
            None,
            None,
        ],
        product: Product::of(1),
    }];
    type Behavior = crate::family::Plain;
}
