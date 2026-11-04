use rspigot_blocks::BlockId;

use crate::{
    AttributeModifier, Craft, EquipmentSlot, Food, Item, ItemId, ItemSet, Product, Rarity, Tool,
};

pub struct WaxedCutCopperStairs;

impl Item for WaxedCutCopperStairs {
    const NAME: &'static str = "minecraft:waxed_cut_copper_stairs";
    const ID: ItemId = ItemId(151);
    const CLASS: &'static str = "BlockItem";
    const MAX_STACK_SIZE: u8 = 64;
    const MAX_DAMAGE: Option<u32> = None;
    const WEAPON_DAMAGE: Option<u32> = None;
    const RARITY: Rarity = Rarity::Common;
    const BLOCK: Option<BlockId> = Some(BlockId(1154));
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
            id: "minecraft:waxed_cut_copper_stairs",
            width: 3,
            height: 3,
            symmetrical: false,
            cells: &[
                Some(ItemSet::Direct(&[ItemId(143)])),
                None,
                None,
                Some(ItemSet::Direct(&[ItemId(143)])),
                Some(ItemSet::Direct(&[ItemId(143)])),
                None,
                Some(ItemSet::Direct(&[ItemId(143)])),
                Some(ItemSet::Direct(&[ItemId(143)])),
                Some(ItemSet::Direct(&[ItemId(143)])),
            ],
            product: Product::of(4),
        },
        Craft::Shapeless {
            id: "minecraft:waxed_cut_copper_stairs_from_honeycomb",
            ingredients: &[ItemSet::Direct(&[ItemId(147)]), ItemSet::Direct(&[ItemId(1530)])],
            product: Product::of(1),
        },
        Craft::Stonecutting {
            id: "minecraft:waxed_cut_copper_stairs_from_waxed_copper_block_stonecutting",
            ingredient: ItemSet::Direct(&[ItemId(124)]),
            count: 4,
        },
        Craft::Stonecutting {
            id: "minecraft:waxed_cut_copper_stairs_from_waxed_cut_copper_stonecutting",
            ingredient: ItemSet::Direct(&[ItemId(143)]),
            count: 1,
        },
    ];
    type Behavior = crate::family::BlockItem;
}
