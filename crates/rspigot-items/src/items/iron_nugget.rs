use rspigot_blocks::BlockId;

use crate::{
    AttributeModifier, CookingKind, Craft, EquipmentSlot, Food, Item, ItemId, ItemSet, Product,
    Rarity, Tool,
};

pub struct IronNugget;

impl Item for IronNugget {
    const NAME: &'static str = "minecraft:iron_nugget";
    const ID: ItemId = ItemId(1456);
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
            ingredients: &[ItemSet::Direct(&[ItemId(1018)])],
            product: Product::of(9),
        },
        Craft::Cooking {
            kind: CookingKind::Blasting,
            ingredient: ItemSet::Direct(&[
                ItemId(1045),
                ItemId(1046),
                ItemId(1047),
                ItemId(1048),
                ItemId(1049),
                ItemId(1076),
                ItemId(1077),
                ItemId(1078),
                ItemId(1079),
                ItemId(1080),
                ItemId(1081),
                ItemId(1082),
                ItemId(1083),
                ItemId(1407),
                ItemId(1450),
                ItemId(1485),
            ]),
            experience: 0.1,
            cooking_time: 200,
        },
        Craft::Cooking {
            kind: CookingKind::Smelting,
            ingredient: ItemSet::Direct(&[
                ItemId(1045),
                ItemId(1046),
                ItemId(1047),
                ItemId(1048),
                ItemId(1049),
                ItemId(1076),
                ItemId(1077),
                ItemId(1078),
                ItemId(1079),
                ItemId(1080),
                ItemId(1081),
                ItemId(1082),
                ItemId(1083),
                ItemId(1407),
                ItemId(1450),
                ItemId(1485),
            ]),
            experience: 0.1,
            cooking_time: 200,
        },
    ];
    type Behavior = crate::family::Plain;
}
