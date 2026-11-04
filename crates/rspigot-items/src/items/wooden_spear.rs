use rspigot_blocks::BlockId;

use crate::{
    AttributeModifier, Craft, EquipmentSlot, Food, Fuel, Item, ItemId, ItemSet, ModifierOperation,
    Product, Rarity, SlotGroup, Tool, tags,
};

pub struct WoodenSpear;

impl Item for WoodenSpear {
    const NAME: &'static str = "minecraft:wooden_spear";
    const ID: ItemId = ItemId(1447);
    const CLASS: &'static str = "Item";
    const MAX_STACK_SIZE: u8 = 1;
    const MAX_DAMAGE: Option<u32> = Some(59);
    const WEAPON_DAMAGE: Option<u32> = Some(1);
    const RARITY: Rarity = Rarity::Common;
    const BLOCK: Option<BlockId> = None;
    const TOOL: Option<Tool> = None;
    const FOOD: Option<Food> = None;
    const EQUIPPABLE: Option<EquipmentSlot> = None;
    const ENCHANTABLE: Option<u32> = Some(15);
    const REPAIRABLE: Option<ItemSet> = Some(ItemSet::Tag(&tags::ITEM_WOODEN_TOOL_MATERIALS));
    const DAMAGE_RESISTANT: Option<&'static str> = None;
    const USE_REMAINDER: Option<ItemId> = None;
    const ATTRIBUTE_MODIFIERS: &'static [AttributeModifier] = &[
        AttributeModifier {
            attribute: "minecraft:attack_damage",
            id: "minecraft:base_attack_damage",
            amount: 0.0,
            operation: ModifierOperation::AddValue,
            slot: SlotGroup::Mainhand,
        },
        AttributeModifier {
            attribute: "minecraft:attack_speed",
            id: "minecraft:base_attack_speed",
            amount: -2.4615384340286255,
            operation: ModifierOperation::AddValue,
            slot: SlotGroup::Mainhand,
        },
    ];
    const COOKING_FUEL: Option<Fuel> = Some(Fuel { burn_time: 200, fast_burn_time: 100 });
    const CRAFTS: &'static [Craft] = &[
        Craft::RepairItem { id: "minecraft:repair_item" },
        Craft::Shaped {
            id: "minecraft:wooden_spear",
            width: 3,
            height: 3,
            symmetrical: false,
            cells: &[
                None,
                None,
                Some(ItemSet::Tag(&tags::ITEM_WOODEN_TOOL_MATERIALS)),
                None,
                Some(ItemSet::Direct(&[ItemId(1060)])),
                None,
                Some(ItemSet::Direct(&[ItemId(1060)])),
                None,
                None,
            ],
            product: Product::of(1),
        },
    ];
    type Behavior = crate::family::Plain;
}
