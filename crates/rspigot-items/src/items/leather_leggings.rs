use rspigot_blocks::BlockId;

use crate::{
    AttributeModifier, Craft, EquipmentSlot, Food, Item, ItemId, ItemSet, ModifierOperation,
    Product, Rarity, SlotGroup, Tool, tags,
};

pub struct LeatherLeggings;

impl Item for LeatherLeggings {
    const NAME: &'static str = "minecraft:leather_leggings";
    const ID: ItemId = ItemId(1070);
    const CLASS: &'static str = "Item";
    const MAX_STACK_SIZE: u8 = 1;
    const MAX_DAMAGE: Option<u32> = Some(75);
    const WEAPON_DAMAGE: Option<u32> = None;
    const RARITY: Rarity = Rarity::Common;
    const BLOCK: Option<BlockId> = None;
    const TOOL: Option<Tool> = None;
    const FOOD: Option<Food> = None;
    const EQUIPPABLE: Option<EquipmentSlot> = Some(EquipmentSlot::Legs);
    const ENCHANTABLE: Option<u32> = Some(15);
    const REPAIRABLE: Option<ItemSet> = Some(ItemSet::Tag(&tags::ITEM_REPAIRS_LEATHER_ARMOR));
    const DAMAGE_RESISTANT: Option<&'static str> = None;
    const USE_REMAINDER: Option<ItemId> = None;
    const ATTRIBUTE_MODIFIERS: &'static [AttributeModifier] = &[
        AttributeModifier {
            attribute: "minecraft:armor",
            id: "minecraft:armor.leggings",
            amount: 2.0,
            operation: ModifierOperation::AddValue,
            slot: SlotGroup::Legs,
        },
        AttributeModifier {
            attribute: "minecraft:armor_toughness",
            id: "minecraft:armor.leggings",
            amount: 0.0,
            operation: ModifierOperation::AddValue,
            slot: SlotGroup::Legs,
        },
    ];
    const CRAFTS: &'static [Craft] = &[
        Craft::Shaped {
            width: 3,
            height: 3,
            symmetrical: true,
            cells: &[
                Some(ItemSet::Direct(&[ItemId(1133)])),
                Some(ItemSet::Direct(&[ItemId(1133)])),
                Some(ItemSet::Direct(&[ItemId(1133)])),
                Some(ItemSet::Direct(&[ItemId(1133)])),
                None,
                Some(ItemSet::Direct(&[ItemId(1133)])),
                Some(ItemSet::Direct(&[ItemId(1133)])),
                None,
                Some(ItemSet::Direct(&[ItemId(1133)])),
            ],
            product: Product::of(1),
        },
        Craft::Dye { dye: ItemSet::Tag(&tags::ITEM_DYES) },
    ];
    type Behavior = crate::family::Plain;
}
