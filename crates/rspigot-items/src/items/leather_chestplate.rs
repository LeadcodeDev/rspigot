use rspigot_blocks::BlockId;

use crate::{
    AttributeModifier, Craft, EquipmentSlot, Food, Item, ItemId, ItemSet, ModifierOperation,
    Product, Rarity, SlotGroup, Tool, tags,
};

pub struct LeatherChestplate;

impl Item for LeatherChestplate {
    const NAME: &'static str = "minecraft:leather_chestplate";
    const ID: ItemId = ItemId(1069);
    const CLASS: &'static str = "Item";
    const MAX_STACK_SIZE: u8 = 1;
    const MAX_DAMAGE: Option<u32> = Some(80);
    const WEAPON_DAMAGE: Option<u32> = None;
    const RARITY: Rarity = Rarity::Common;
    const BLOCK: Option<BlockId> = None;
    const TOOL: Option<Tool> = None;
    const FOOD: Option<Food> = None;
    const EQUIPPABLE: Option<EquipmentSlot> = Some(EquipmentSlot::Chest);
    const ENCHANTABLE: Option<u32> = Some(15);
    const REPAIRABLE: Option<ItemSet> = Some(ItemSet::Tag(&tags::ITEM_REPAIRS_LEATHER_ARMOR));
    const DAMAGE_RESISTANT: Option<&'static str> = None;
    const USE_REMAINDER: Option<ItemId> = None;
    const ATTRIBUTE_MODIFIERS: &'static [AttributeModifier] = &[
        AttributeModifier {
            attribute: "minecraft:armor",
            id: "minecraft:armor.chestplate",
            amount: 3.0,
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
            id: "minecraft:leather_chestplate",
            width: 3,
            height: 3,
            symmetrical: true,
            cells: &[
                Some(ItemSet::Direct(&[ItemId(1133)])),
                None,
                Some(ItemSet::Direct(&[ItemId(1133)])),
                Some(ItemSet::Direct(&[ItemId(1133)])),
                Some(ItemSet::Direct(&[ItemId(1133)])),
                Some(ItemSet::Direct(&[ItemId(1133)])),
                Some(ItemSet::Direct(&[ItemId(1133)])),
                Some(ItemSet::Direct(&[ItemId(1133)])),
                Some(ItemSet::Direct(&[ItemId(1133)])),
            ],
            product: Product::of(1),
        },
        Craft::Dye { id: "minecraft:leather_chestplate_dyed", dye: ItemSet::Tag(&tags::ITEM_DYES) },
        Craft::RepairItem { id: "minecraft:repair_item" },
    ];
    type Behavior = crate::family::Plain;
}
