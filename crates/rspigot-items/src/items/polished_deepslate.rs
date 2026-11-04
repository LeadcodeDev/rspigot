use rspigot_blocks::BlockId;

use crate::{
    AttributeModifier, Craft, EquipmentSlot, Food, Item, ItemId, ItemSet, Product, Rarity, Tool,
};

pub struct PolishedDeepslate;

impl Item for PolishedDeepslate {
    const NAME: &'static str = "minecraft:polished_deepslate";
    const ID: ItemId = ItemId(10);
    const CLASS: &'static str = "BlockItem";
    const MAX_STACK_SIZE: u8 = 64;
    const MAX_DAMAGE: Option<u32> = None;
    const WEAPON_DAMAGE: Option<u32> = None;
    const RARITY: Rarity = Rarity::Common;
    const BLOCK: Option<BlockId> = Some(BlockId(1246));
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
            id: "minecraft:polished_deepslate",
            width: 2,
            height: 2,
            symmetrical: true,
            cells: &[
                Some(ItemSet::Direct(&[ItemId(9)])),
                Some(ItemSet::Direct(&[ItemId(9)])),
                Some(ItemSet::Direct(&[ItemId(9)])),
                Some(ItemSet::Direct(&[ItemId(9)])),
            ],
            product: Product::of(4),
        },
        Craft::Stonecutting {
            id: "minecraft:polished_deepslate_from_cobbled_deepslate_stonecutting",
            ingredient: ItemSet::Direct(&[ItemId(9)]),
            count: 1,
        },
        Craft::Stonecutting {
            id: "minecraft:polished_deepslate_from_deepslate_stonecutting",
            ingredient: ItemSet::Direct(&[ItemId(8)]),
            count: 1,
        },
    ];
    type Behavior = crate::family::BlockItem;
}
