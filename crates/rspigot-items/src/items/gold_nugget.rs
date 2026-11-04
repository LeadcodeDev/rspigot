use rspigot_blocks::BlockId;

use crate::{
    AttributeModifier, CookingKind, Craft, EquipmentSlot, Food, Item, ItemId, ItemSet, Product,
    Rarity, Tool,
};

pub struct GoldNugget;

impl Item for GoldNugget {
    const NAME: &'static str = "minecraft:gold_nugget";
    const ID: ItemId = ItemId(1268);
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
            id: "minecraft:gold_nugget",
            ingredients: &[ItemSet::Direct(&[ItemId(1022)])],
            product: Product::of(9),
        },
        Craft::Cooking {
            id: "minecraft:gold_nugget_from_blasting",
            kind: CookingKind::Blasting,
            ingredient: ItemSet::Direct(&[
                ItemId(1040),
                ItemId(1041),
                ItemId(1042),
                ItemId(1043),
                ItemId(1044),
                ItemId(1088),
                ItemId(1089),
                ItemId(1090),
                ItemId(1091),
                ItemId(1408),
                ItemId(1451),
                ItemId(1486),
            ]),
            experience: 0.1,
            cooking_time: 200,
        },
        Craft::Cooking {
            id: "minecraft:gold_nugget_from_smelting",
            kind: CookingKind::Smelting,
            ingredient: ItemSet::Direct(&[
                ItemId(1040),
                ItemId(1041),
                ItemId(1042),
                ItemId(1043),
                ItemId(1044),
                ItemId(1088),
                ItemId(1089),
                ItemId(1090),
                ItemId(1091),
                ItemId(1408),
                ItemId(1451),
                ItemId(1486),
            ]),
            experience: 0.1,
            cooking_time: 200,
        },
    ];
    type Behavior = crate::family::Plain;
}
