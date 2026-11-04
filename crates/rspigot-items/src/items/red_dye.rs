use rspigot_blocks::BlockId;

use crate::{
    AttributeModifier, Craft, EquipmentSlot, Food, Item, ItemId, ItemSet, Product, Rarity, Tool,
};

pub struct RedDye;

impl Item for RedDye {
    const NAME: &'static str = "minecraft:red_dye";
    const ID: ItemId = ItemId(1213);
    const CLASS: &'static str = "DyeItem";
    const MAX_STACK_SIZE: u8 = 64;
    const MAX_DAMAGE: Option<u32> = None;
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
        Craft::Shapeless {
            id: "minecraft:red_dye_from_beetroot",
            ingredients: &[ItemSet::Direct(&[ItemId(1438)])],
            product: Product::of(1),
        },
        Craft::Shapeless {
            id: "minecraft:red_dye_from_poppy",
            ingredients: &[ItemSet::Direct(&[ItemId(302)])],
            product: Product::of(1),
        },
        Craft::Shapeless {
            id: "minecraft:red_dye_from_rose_bush",
            ingredients: &[ItemSet::Direct(&[ItemId(601)])],
            product: Product::of(2),
        },
        Craft::Shapeless {
            id: "minecraft:red_dye_from_tulip",
            ingredients: &[ItemSet::Direct(&[ItemId(306)])],
            product: Product::of(1),
        },
    ];
    type Behavior = crate::family::Plain;
}
