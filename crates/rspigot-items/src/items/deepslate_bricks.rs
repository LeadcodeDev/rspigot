use rspigot_blocks::BlockId;

use crate::{
    AttributeModifier, Craft, EquipmentSlot, Food, Item, ItemId, ItemSet, Product, Rarity, Tool,
};

pub struct DeepslateBricks;

impl Item for DeepslateBricks {
    const NAME: &'static str = "minecraft:deepslate_bricks";
    const ID: ItemId = ItemId(455);
    const CLASS: &'static str = "BlockItem";
    const MAX_STACK_SIZE: u8 = 64;
    const MAX_DAMAGE: Option<u32> = None;
    const WEAPON_DAMAGE: Option<u32> = None;
    const RARITY: Rarity = Rarity::Common;
    const BLOCK: Option<BlockId> = Some(BlockId(1254));
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
            width: 2,
            height: 2,
            symmetrical: true,
            cells: &[
                Some(ItemSet::Direct(&[ItemId(10)])),
                Some(ItemSet::Direct(&[ItemId(10)])),
                Some(ItemSet::Direct(&[ItemId(10)])),
                Some(ItemSet::Direct(&[ItemId(10)])),
            ],
            product: Product::of(4),
        },
        Craft::Stonecutting {
            id: "minecraft:deepslate_bricks_from_cobbled_deepslate_stonecutting",
            ingredient: ItemSet::Direct(&[ItemId(9)]),
            count: 1,
        },
        Craft::Stonecutting {
            id: "minecraft:deepslate_bricks_from_deepslate_stonecutting",
            ingredient: ItemSet::Direct(&[ItemId(8)]),
            count: 1,
        },
        Craft::Stonecutting {
            id: "minecraft:deepslate_bricks_from_polished_deepslate_stonecutting",
            ingredient: ItemSet::Direct(&[ItemId(10)]),
            count: 1,
        },
    ];
    type Behavior = crate::family::BlockItem;
}
