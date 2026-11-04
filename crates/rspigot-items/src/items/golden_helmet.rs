use rspigot_blocks::BlockId;

use crate::{
    AttributeModifier, Craft, EquipmentSlot, Food, Item, ItemId, ItemSet, ModifierOperation,
    Product, Rarity, SlotGroup, Tool, tags,
};

pub struct GoldenHelmet;

impl Item for GoldenHelmet {
    const NAME: &'static str = "minecraft:golden_helmet";
    const ID: ItemId = ItemId(1088);
    const CLASS: &'static str = "Item";
    const MAX_STACK_SIZE: u8 = 1;
    const MAX_DAMAGE: Option<u32> = Some(77);
    const WEAPON_DAMAGE: Option<u32> = None;
    const RARITY: Rarity = Rarity::Common;
    const BLOCK: Option<BlockId> = None;
    const TOOL: Option<Tool> = None;
    const FOOD: Option<Food> = None;
    const EQUIPPABLE: Option<EquipmentSlot> = Some(EquipmentSlot::Head);
    const ENCHANTABLE: Option<u32> = Some(25);
    const REPAIRABLE: Option<ItemSet> = Some(ItemSet::Tag(&tags::ITEM_REPAIRS_GOLD_ARMOR));
    const DAMAGE_RESISTANT: Option<&'static str> = None;
    const USE_REMAINDER: Option<ItemId> = None;
    const ATTRIBUTE_MODIFIERS: &'static [AttributeModifier] = &[
        AttributeModifier {
            attribute: "minecraft:armor",
            id: "minecraft:armor.helmet",
            amount: 2.0,
            operation: ModifierOperation::AddValue,
            slot: SlotGroup::Head,
        },
        AttributeModifier {
            attribute: "minecraft:armor_toughness",
            id: "minecraft:armor.helmet",
            amount: 0.0,
            operation: ModifierOperation::AddValue,
            slot: SlotGroup::Head,
        },
    ];
    const CRAFTS: &'static [Craft] = &[
        Craft::Shaped {
            id: "minecraft:golden_helmet",
            width: 3,
            height: 2,
            symmetrical: true,
            cells: &[
                Some(ItemSet::Direct(&[ItemId(1022)])),
                Some(ItemSet::Direct(&[ItemId(1022)])),
                Some(ItemSet::Direct(&[ItemId(1022)])),
                Some(ItemSet::Direct(&[ItemId(1022)])),
                None,
                Some(ItemSet::Direct(&[ItemId(1022)])),
            ],
            product: Product::of(1),
        },
        Craft::RepairItem { id: "minecraft:repair_item" },
    ];
    type Behavior = crate::family::Plain;
}
