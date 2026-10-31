use rspigot_blocks::BlockId;

use crate::{
    AttributeModifier, Craft, EquipmentSlot, Food, Item, ItemId, ItemSet, Product, Rarity, Tool,
};

pub struct WaxedExposedCutCopper;

impl Item for WaxedExposedCutCopper {
    const NAME: &'static str = "minecraft:waxed_exposed_cut_copper";
    const ID: ItemId = ItemId(144);
    const CLASS: &'static str = "BlockItem";
    const MAX_STACK_SIZE: u8 = 64;
    const MAX_DAMAGE: Option<u32> = None;
    const WEAPON_DAMAGE: Option<u32> = None;
    const RARITY: Rarity = Rarity::Common;
    const BLOCK: Option<BlockId> = Some(BlockId(1139));
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
                Some(ItemSet::Direct(&[ItemId(125)])),
                Some(ItemSet::Direct(&[ItemId(125)])),
                Some(ItemSet::Direct(&[ItemId(125)])),
                Some(ItemSet::Direct(&[ItemId(125)])),
            ],
            product: Product::of(4),
        },
        Craft::Shapeless {
            ingredients: &[ItemSet::Direct(&[ItemId(140)]), ItemSet::Direct(&[ItemId(1530)])],
            product: Product::of(1),
        },
        Craft::Stonecutting {
            id: "minecraft:waxed_exposed_cut_copper_from_waxed_exposed_copper_stonecutting",
            ingredient: ItemSet::Direct(&[ItemId(125)]),
            count: 4,
        },
    ];
    type Behavior = crate::family::BlockItem;
}
