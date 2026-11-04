use rspigot_blocks::BlockId;

use crate::{
    AttributeModifier, Craft, EquipmentSlot, Food, Item, ItemId, ItemSet, ModifierOperation,
    Product, Rarity, SlotGroup, Tool, tags,
};

pub struct DiamondLeggings;

impl Item for DiamondLeggings {
    const NAME: &'static str = "minecraft:diamond_leggings";
    const ID: ItemId = ItemId(1086);
    const CLASS: &'static str = "Item";
    const MAX_STACK_SIZE: u8 = 1;
    const MAX_DAMAGE: Option<u32> = Some(495);
    const WEAPON_DAMAGE: Option<u32> = None;
    const RARITY: Rarity = Rarity::Common;
    const BLOCK: Option<BlockId> = None;
    const TOOL: Option<Tool> = None;
    const FOOD: Option<Food> = None;
    const EQUIPPABLE: Option<EquipmentSlot> = Some(EquipmentSlot::Legs);
    const ENCHANTABLE: Option<u32> = Some(10);
    const REPAIRABLE: Option<ItemSet> = Some(ItemSet::Tag(&tags::ITEM_REPAIRS_DIAMOND_ARMOR));
    const DAMAGE_RESISTANT: Option<&'static str> = None;
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
            amount: 2.0,
            operation: ModifierOperation::AddValue,
            slot: SlotGroup::Legs,
        },
    ];
    const CRAFTS: &'static [Craft] = &[
        Craft::Shaped {
            id: "minecraft:diamond_leggings",
            width: 3,
            height: 3,
            symmetrical: true,
            cells: &[
                Some(ItemSet::Direct(&[ItemId(1012)])),
                Some(ItemSet::Direct(&[ItemId(1012)])),
                Some(ItemSet::Direct(&[ItemId(1012)])),
                Some(ItemSet::Direct(&[ItemId(1012)])),
                None,
                Some(ItemSet::Direct(&[ItemId(1012)])),
                Some(ItemSet::Direct(&[ItemId(1012)])),
                None,
                Some(ItemSet::Direct(&[ItemId(1012)])),
            ],
            product: Product::of(1),
        },
        Craft::RepairItem { id: "minecraft:repair_item" },
    ];
    type Behavior = crate::family::Plain;
}
