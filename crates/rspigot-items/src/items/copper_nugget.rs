use rspigot_blocks::BlockId;

use crate::{
    AttributeModifier, CookingKind, Craft, EquipmentSlot, Food, Item, ItemId, ItemSet, Product,
    Rarity, Tool,
};

pub struct CopperNugget;

impl Item for CopperNugget {
    const NAME: &'static str = "minecraft:copper_nugget";
    const ID: ItemId = ItemId(1457);
    const CLASS: &'static str = "Item";
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
            ingredients: &[ItemSet::Direct(&[ItemId(1020)])],
            product: Product::of(9),
        },
        Craft::Cooking {
            id: "minecraft:copper_nugget_from_blasting",
            kind: CookingKind::Blasting,
            ingredient: ItemSet::Direct(&[
                ItemId(1030),
                ItemId(1031),
                ItemId(1032),
                ItemId(1033),
                ItemId(1034),
                ItemId(1072),
                ItemId(1073),
                ItemId(1074),
                ItemId(1075),
                ItemId(1406),
                ItemId(1449),
                ItemId(1489),
            ]),
            experience: 0.1,
            cooking_time: 200,
        },
        Craft::Cooking {
            id: "minecraft:copper_nugget_from_smelting",
            kind: CookingKind::Smelting,
            ingredient: ItemSet::Direct(&[
                ItemId(1030),
                ItemId(1031),
                ItemId(1032),
                ItemId(1033),
                ItemId(1034),
                ItemId(1072),
                ItemId(1073),
                ItemId(1074),
                ItemId(1075),
                ItemId(1406),
                ItemId(1449),
                ItemId(1489),
            ]),
            experience: 0.1,
            cooking_time: 200,
        },
    ];
    type Behavior = crate::family::Plain;
}
