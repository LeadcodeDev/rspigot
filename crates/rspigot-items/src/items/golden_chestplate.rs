use rspigot_blocks::BlockId;

use crate::{
    AttributeModifier, Craft, EquipmentSlot, Food, Item, ItemId, ItemSet, ModifierOperation,
    Product, Rarity, SlotGroup, Tool, tags,
};

pub struct GoldenChestplate;

impl Item for GoldenChestplate {
    const NAME: &'static str = "minecraft:golden_chestplate";
    const ID: ItemId = ItemId(1089);
    const CLASS: &'static str = "Item";
    const MAX_STACK_SIZE: u8 = 1;
    const MAX_DAMAGE: Option<u32> = Some(112);
    const WEAPON_DAMAGE: Option<u32> = None;
    const RARITY: Rarity = Rarity::Common;
    const BLOCK: Option<BlockId> = None;
    const TOOL: Option<Tool> = None;
    const FOOD: Option<Food> = None;
    const EQUIPPABLE: Option<EquipmentSlot> = Some(EquipmentSlot::Chest);
    const ENCHANTABLE: Option<u32> = Some(25);
    const REPAIRABLE: Option<ItemSet> = Some(ItemSet::Tag(&tags::ITEM_REPAIRS_GOLD_ARMOR));
    const DAMAGE_RESISTANT: Option<&'static str> = None;
    const USE_REMAINDER: Option<ItemId> = None;
    const ATTRIBUTE_MODIFIERS: &'static [AttributeModifier] = &[
        AttributeModifier {
            attribute: "minecraft:armor",
            id: "minecraft:armor.chestplate",
            amount: 5.0,
            operation: ModifierOperation::AddValue,
            slot: SlotGroup::Chest,
        },
        AttributeModifier {
            attribute: "minecraft:armor_toughness",
            id: "minecraft:armor.chestplate",
            amount: 0.0,
            operation: ModifierOperation::AddValue,
            slot: SlotGroup::Chest,
        },
    ];
    const CRAFTS: &'static [Craft] = &[
        Craft::Shaped {
            id: "minecraft:golden_chestplate",
            width: 3,
            height: 3,
            symmetrical: true,
            cells: &[
                Some(ItemSet::Direct(&[ItemId(1022)])),
                None,
                Some(ItemSet::Direct(&[ItemId(1022)])),
                Some(ItemSet::Direct(&[ItemId(1022)])),
                Some(ItemSet::Direct(&[ItemId(1022)])),
                Some(ItemSet::Direct(&[ItemId(1022)])),
                Some(ItemSet::Direct(&[ItemId(1022)])),
                Some(ItemSet::Direct(&[ItemId(1022)])),
                Some(ItemSet::Direct(&[ItemId(1022)])),
            ],
            product: Product::of(1),
        },
        Craft::RepairItem { id: "minecraft:repair_item" },
    ];
    type Behavior = crate::family::Plain;
}
