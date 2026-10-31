use rspigot_blocks::BlockId;

use crate::{
    AttributeModifier, Craft, EquipmentSlot, Food, Item, ItemId, ItemSet, ModifierOperation,
    Product, Rarity, SlotGroup, Tool, tags,
};

pub struct DiamondBoots;

impl Item for DiamondBoots {
    const NAME: &'static str = "minecraft:diamond_boots";
    const ID: ItemId = ItemId(1087);
    const CLASS: &'static str = "Item";
    const MAX_STACK_SIZE: u8 = 1;
    const MAX_DAMAGE: Option<u32> = Some(429);
    const WEAPON_DAMAGE: Option<u32> = None;
    const RARITY: Rarity = Rarity::Common;
    const BLOCK: Option<BlockId> = None;
    const TOOL: Option<Tool> = None;
    const FOOD: Option<Food> = None;
    const EQUIPPABLE: Option<EquipmentSlot> = Some(EquipmentSlot::Feet);
    const ENCHANTABLE: Option<u32> = Some(10);
    const REPAIRABLE: Option<ItemSet> = Some(ItemSet::Tag(&tags::ITEM_REPAIRS_DIAMOND_ARMOR));
    const DAMAGE_RESISTANT: Option<&'static str> = None;
    const USE_REMAINDER: Option<ItemId> = None;
    const ATTRIBUTE_MODIFIERS: &'static [AttributeModifier] = &[
        AttributeModifier {
            attribute: "minecraft:armor",
            id: "minecraft:armor.boots",
            amount: 3.0,
            operation: ModifierOperation::AddValue,
            slot: SlotGroup::Feet,
        },
        AttributeModifier {
            attribute: "minecraft:armor_toughness",
            id: "minecraft:armor.boots",
            amount: 2.0,
            operation: ModifierOperation::AddValue,
            slot: SlotGroup::Feet,
        },
    ];
    const CRAFTS: &'static [Craft] = &[Craft::Shaped {
        width: 3,
        height: 2,
        symmetrical: true,
        cells: &[
            Some(ItemSet::Direct(&[ItemId(1012)])),
            None,
            Some(ItemSet::Direct(&[ItemId(1012)])),
            Some(ItemSet::Direct(&[ItemId(1012)])),
            None,
            Some(ItemSet::Direct(&[ItemId(1012)])),
        ],
        product: Product::of(1),
    }];
    type Behavior = crate::family::Plain;
}
