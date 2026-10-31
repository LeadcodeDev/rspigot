use rspigot_blocks::BlockId;

use crate::{
    AttributeModifier, Craft, EquipmentSlot, Food, Item, ItemId, ItemSet, Product, Rarity, Tool,
};

pub struct PolishedCinnabarSlab;

impl Item for PolishedCinnabarSlab {
    const NAME: &'static str = "minecraft:polished_cinnabar_slab";
    const ID: ItemId = ItemId(45);
    const CLASS: &'static str = "BlockItem";
    const MAX_STACK_SIZE: u8 = 64;
    const MAX_DAMAGE: Option<u32> = None;
    const WEAPON_DAMAGE: Option<u32> = None;
    const RARITY: Rarity = Rarity::Common;
    const BLOCK: Option<BlockId> = Some(BlockId(1107));
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
            width: 3,
            height: 1,
            symmetrical: true,
            cells: &[
                Some(ItemSet::Direct(&[ItemId(44)])),
                Some(ItemSet::Direct(&[ItemId(44)])),
                Some(ItemSet::Direct(&[ItemId(44)])),
            ],
            product: Product::of(6),
        },
        Craft::Stonecutting { ingredient: ItemSet::Direct(&[ItemId(40)]), count: 2 },
        Craft::Stonecutting { ingredient: ItemSet::Direct(&[ItemId(44)]), count: 2 },
    ];
    type Behavior = crate::family::BlockItem;
}
