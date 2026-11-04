use rspigot_blocks::BlockId;

use crate::{
    AttributeModifier, Craft, EquipmentSlot, Food, Item, ItemId, ItemSet, Product, Rarity, Tool,
};

pub struct PolishedGranite;

impl Item for PolishedGranite {
    const NAME: &'static str = "minecraft:polished_granite";
    const ID: ItemId = ItemId(3);
    const CLASS: &'static str = "BlockItem";
    const MAX_STACK_SIZE: u8 = 64;
    const MAX_DAMAGE: Option<u32> = None;
    const WEAPON_DAMAGE: Option<u32> = None;
    const RARITY: Rarity = Rarity::Common;
    const BLOCK: Option<BlockId> = Some(BlockId(3));
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
            id: "minecraft:polished_granite",
            width: 2,
            height: 2,
            symmetrical: true,
            cells: &[
                Some(ItemSet::Direct(&[ItemId(2)])),
                Some(ItemSet::Direct(&[ItemId(2)])),
                Some(ItemSet::Direct(&[ItemId(2)])),
                Some(ItemSet::Direct(&[ItemId(2)])),
            ],
            product: Product::of(4),
        },
        Craft::Stonecutting {
            id: "minecraft:polished_granite_from_granite_stonecutting",
            ingredient: ItemSet::Direct(&[ItemId(2)]),
            count: 1,
        },
    ];
    type Behavior = crate::family::BlockItem;
}
