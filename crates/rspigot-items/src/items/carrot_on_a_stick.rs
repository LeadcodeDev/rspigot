use rspigot_blocks::BlockId;

use crate::{
    AttributeModifier, Craft, EquipmentSlot, Food, Item, ItemId, ItemSet, Product, Rarity, Tool,
};

pub struct CarrotOnAStick;

impl Item for CarrotOnAStick {
    const NAME: &'static str = "minecraft:carrot_on_a_stick";
    const ID: ItemId = ItemId(971);
    const CLASS: &'static str = "FoodOnAStickItem";
    const MAX_STACK_SIZE: u8 = 1;
    const MAX_DAMAGE: Option<u32> = Some(25);
    const WEAPON_DAMAGE: Option<u32> = None;
    const RARITY: Rarity = Rarity::Common;
    const BLOCK: Option<BlockId> = None;
    const TOOL: Option<Tool> = None;
    const FOOD: Option<Food> = None;
    const EQUIPPABLE: Option<EquipmentSlot> = None;
    const ENCHANTABLE: Option<u32> = None;
    const REPAIRABLE: Option<ItemSet> = None;
    const DAMAGE_RESISTANT: Option<&'static str> = None;
    const USE_REMAINDER: Option<ItemId> = None;
    const ATTRIBUTE_MODIFIERS: &'static [AttributeModifier] = &[];
    const CRAFTS: &'static [Craft] = &[
        Craft::Shaped {
            id: "minecraft:carrot_on_a_stick",
            width: 2,
            height: 2,
            symmetrical: false,
            cells: &[
                Some(ItemSet::Direct(&[ItemId(1186)])),
                None,
                None,
                Some(ItemSet::Direct(&[ItemId(1378)])),
            ],
            product: Product::of(1),
        },
        Craft::RepairItem { id: "minecraft:repair_item" },
    ];
    type Behavior = crate::family::Plain;
}
