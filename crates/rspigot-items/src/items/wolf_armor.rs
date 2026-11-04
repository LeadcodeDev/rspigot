use rspigot_blocks::BlockId;

use crate::{
    AttributeModifier, Craft, EquipmentSlot, Food, Item, ItemId, ItemSet, ModifierOperation,
    Product, Rarity, SlotGroup, Tool, tags,
};

pub struct WolfArmor;

impl Item for WolfArmor {
    const NAME: &'static str = "minecraft:wolf_armor";
    const ID: ItemId = ItemId(1004);
    const CLASS: &'static str = "Item";
    const MAX_STACK_SIZE: u8 = 1;
    const MAX_DAMAGE: Option<u32> = Some(64);
    const WEAPON_DAMAGE: Option<u32> = None;
    const RARITY: Rarity = Rarity::Common;
    const BLOCK: Option<BlockId> = None;
    const TOOL: Option<Tool> = None;
    const FOOD: Option<Food> = None;
    const EQUIPPABLE: Option<EquipmentSlot> = Some(EquipmentSlot::Body);
    const ENCHANTABLE: Option<u32> = None;
    const REPAIRABLE: Option<ItemSet> = Some(ItemSet::Tag(&tags::ITEM_REPAIRS_WOLF_ARMOR));
    const DAMAGE_RESISTANT: Option<&'static str> = None;
    const USE_REMAINDER: Option<ItemId> = None;
    const ATTRIBUTE_MODIFIERS: &'static [AttributeModifier] = &[
        AttributeModifier {
            attribute: "minecraft:armor",
            id: "minecraft:armor.body",
            amount: 11.0,
            operation: ModifierOperation::AddValue,
            slot: SlotGroup::Body,
        },
        AttributeModifier {
            attribute: "minecraft:armor_toughness",
            id: "minecraft:armor.body",
            amount: 0.0,
            operation: ModifierOperation::AddValue,
            slot: SlotGroup::Body,
        },
    ];
    const CRAFTS: &'static [Craft] = &[
        Craft::RepairItem { id: "minecraft:repair_item" },
        Craft::Shaped {
            id: "minecraft:wolf_armor",
            width: 3,
            height: 3,
            symmetrical: false,
            cells: &[
                Some(ItemSet::Direct(&[ItemId(1003)])),
                None,
                None,
                Some(ItemSet::Direct(&[ItemId(1003)])),
                Some(ItemSet::Direct(&[ItemId(1003)])),
                Some(ItemSet::Direct(&[ItemId(1003)])),
                Some(ItemSet::Direct(&[ItemId(1003)])),
                None,
                Some(ItemSet::Direct(&[ItemId(1003)])),
            ],
            product: Product::of(1),
        },
        Craft::Dye { id: "minecraft:wolf_armor_dyed", dye: ItemSet::Tag(&tags::ITEM_DYES) },
    ];
    type Behavior = crate::family::Plain;
}
