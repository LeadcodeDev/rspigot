use rspigot_blocks::BlockId;

use crate::{
    AttributeModifier, Craft, EquipmentSlot, Food, Item, ItemId, ItemSet, Product, Rarity, Tool,
};

pub struct PolishedDeepslateWall;

impl Item for PolishedDeepslateWall {
    const NAME: &'static str = "minecraft:polished_deepslate_wall";
    const ID: ItemId = ItemId(550);
    const CLASS: &'static str = "BlockItem";
    const MAX_STACK_SIZE: u8 = 64;
    const MAX_DAMAGE: Option<u32> = None;
    const WEAPON_DAMAGE: Option<u32> = None;
    const RARITY: Rarity = Rarity::Common;
    const BLOCK: Option<BlockId> = Some(BlockId(1249));
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
            height: 2,
            symmetrical: true,
            cells: &[
                Some(ItemSet::Direct(&[ItemId(10)])),
                Some(ItemSet::Direct(&[ItemId(10)])),
                Some(ItemSet::Direct(&[ItemId(10)])),
                Some(ItemSet::Direct(&[ItemId(10)])),
                Some(ItemSet::Direct(&[ItemId(10)])),
                Some(ItemSet::Direct(&[ItemId(10)])),
            ],
            product: Product::of(6),
        },
        Craft::Stonecutting { ingredient: ItemSet::Direct(&[ItemId(9)]), count: 1 },
        Craft::Stonecutting { ingredient: ItemSet::Direct(&[ItemId(8)]), count: 1 },
        Craft::Stonecutting { ingredient: ItemSet::Direct(&[ItemId(10)]), count: 1 },
    ];
    type Behavior = crate::family::BlockItem;
}
