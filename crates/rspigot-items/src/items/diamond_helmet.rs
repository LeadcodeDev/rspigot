use rspigot_blocks::BlockId;

use crate::{
    AttributeModifier, Craft, EquipmentSlot, Food, Item, ItemId, ItemSet, ModifierOperation,
    Product, Rarity, SlotGroup, Tool, tags,
};

pub struct DiamondHelmet;

impl Item for DiamondHelmet {
    const NAME: &'static str = "minecraft:diamond_helmet";
    const ID: ItemId = ItemId(1084);
    const CLASS: &'static str = "Item";
    const MAX_STACK_SIZE: u8 = 1;
    const MAX_DAMAGE: Option<u32> = Some(363);
    const WEAPON_DAMAGE: Option<u32> = None;
    const RARITY: Rarity = Rarity::Common;
    const BLOCK: Option<BlockId> = None;
    const TOOL: Option<Tool> = None;
    const FOOD: Option<Food> = None;
    const EQUIPPABLE: Option<EquipmentSlot> = Some(EquipmentSlot::Head);
    const ENCHANTABLE: Option<u32> = Some(10);
    const REPAIRABLE: Option<ItemSet> = Some(ItemSet::Tag(&tags::ITEM_REPAIRS_DIAMOND_ARMOR));
    const DAMAGE_RESISTANT: Option<&'static str> = None;
    const USE_REMAINDER: Option<ItemId> = None;
    const ATTRIBUTE_MODIFIERS: &'static [AttributeModifier] = &[
        AttributeModifier {
            attribute: "minecraft:armor",
            id: "minecraft:armor.helmet",
            amount: 3.0,
            operation: ModifierOperation::AddValue,
            slot: SlotGroup::Head,
        },
        AttributeModifier {
            attribute: "minecraft:armor_toughness",
            id: "minecraft:armor.helmet",
            amount: 2.0,
            operation: ModifierOperation::AddValue,
            slot: SlotGroup::Head,
        },
    ];
    const CRAFTS: &'static [Craft] = &[
        Craft::Shaped {
            id: "minecraft:diamond_helmet",
            width: 3,
            height: 2,
            symmetrical: true,
            cells: &[
                Some(ItemSet::Direct(&[ItemId(1012)])),
                Some(ItemSet::Direct(&[ItemId(1012)])),
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
