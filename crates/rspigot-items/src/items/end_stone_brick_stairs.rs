use rspigot_blocks::BlockId;

use crate::{
    AttributeModifier, Craft, EquipmentSlot, Food, Item, ItemId, ItemSet, Product, Rarity, Tool,
};

pub struct EndStoneBrickStairs;

impl Item for EndStoneBrickStairs {
    const NAME: &'static str = "minecraft:end_stone_brick_stairs";
    const ID: ItemId = ItemId(793);
    const CLASS: &'static str = "BlockItem";
    const MAX_STACK_SIZE: u8 = 64;
    const MAX_DAMAGE: Option<u32> = None;
    const WEAPON_DAMAGE: Option<u32> = None;
    const RARITY: Rarity = Rarity::Common;
    const BLOCK: Option<BlockId> = Some(BlockId(892));
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
            id: "minecraft:end_stone_brick_stairs",
            width: 3,
            height: 3,
            symmetrical: false,
            cells: &[
                Some(ItemSet::Direct(&[ItemId(510)])),
                None,
                None,
                Some(ItemSet::Direct(&[ItemId(510)])),
                Some(ItemSet::Direct(&[ItemId(510)])),
                None,
                Some(ItemSet::Direct(&[ItemId(510)])),
                Some(ItemSet::Direct(&[ItemId(510)])),
                Some(ItemSet::Direct(&[ItemId(510)])),
            ],
            product: Product::of(4),
        },
        Craft::Stonecutting {
            id: "minecraft:end_stone_brick_stairs_from_end_stone_bricks_stonecutting",
            ingredient: ItemSet::Direct(&[ItemId(510)]),
            count: 1,
        },
        Craft::Stonecutting {
            id: "minecraft:end_stone_brick_stairs_from_end_stone_stonecutting",
            ingredient: ItemSet::Direct(&[ItemId(509)]),
            count: 1,
        },
    ];
    type Behavior = crate::family::BlockItem;
}
