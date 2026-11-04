use rspigot_blocks::BlockId;

use crate::{
    AttributeModifier, Craft, EquipmentSlot, Food, Item, ItemId, ItemSet, Product, Rarity, Tool,
};

pub struct DeepslateTileStairs;

impl Item for DeepslateTileStairs {
    const NAME: &'static str = "minecraft:deepslate_tile_stairs";
    const ID: ItemId = ItemId(805);
    const CLASS: &'static str = "BlockItem";
    const MAX_STACK_SIZE: u8 = 64;
    const MAX_DAMAGE: Option<u32> = None;
    const WEAPON_DAMAGE: Option<u32> = None;
    const RARITY: Rarity = Rarity::Common;
    const BLOCK: Option<BlockId> = Some(BlockId(1251));
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
            id: "minecraft:deepslate_tile_stairs",
            width: 3,
            height: 3,
            symmetrical: false,
            cells: &[
                Some(ItemSet::Direct(&[ItemId(457)])),
                None,
                None,
                Some(ItemSet::Direct(&[ItemId(457)])),
                Some(ItemSet::Direct(&[ItemId(457)])),
                None,
                Some(ItemSet::Direct(&[ItemId(457)])),
                Some(ItemSet::Direct(&[ItemId(457)])),
                Some(ItemSet::Direct(&[ItemId(457)])),
            ],
            product: Product::of(4),
        },
        Craft::Stonecutting {
            id: "minecraft:deepslate_tile_stairs_from_cobbled_deepslate_stonecutting",
            ingredient: ItemSet::Direct(&[ItemId(9)]),
            count: 1,
        },
        Craft::Stonecutting {
            id: "minecraft:deepslate_tile_stairs_from_deepslate_bricks_stonecutting",
            ingredient: ItemSet::Direct(&[ItemId(455)]),
            count: 1,
        },
        Craft::Stonecutting {
            id: "minecraft:deepslate_tile_stairs_from_deepslate_stonecutting",
            ingredient: ItemSet::Direct(&[ItemId(8)]),
            count: 1,
        },
        Craft::Stonecutting {
            id: "minecraft:deepslate_tile_stairs_from_deepslate_tiles_stonecutting",
            ingredient: ItemSet::Direct(&[ItemId(457)]),
            count: 1,
        },
        Craft::Stonecutting {
            id: "minecraft:deepslate_tile_stairs_from_polished_deepslate_stonecutting",
            ingredient: ItemSet::Direct(&[ItemId(10)]),
            count: 1,
        },
    ];
    type Behavior = crate::family::BlockItem;
}
