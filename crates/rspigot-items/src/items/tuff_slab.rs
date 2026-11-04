use rspigot_blocks::BlockId;

use crate::{
    AttributeModifier, Craft, EquipmentSlot, Food, Item, ItemId, ItemSet, Product, Rarity, Tool,
};

pub struct TuffSlab;

impl Item for TuffSlab {
    const NAME: &'static str = "minecraft:tuff_slab";
    const ID: ItemId = ItemId(13);
    const CLASS: &'static str = "BlockItem";
    const MAX_STACK_SIZE: u8 = 64;
    const MAX_DAMAGE: Option<u32> = None;
    const WEAPON_DAMAGE: Option<u32> = None;
    const RARITY: Rarity = Rarity::Common;
    const BLOCK: Option<BlockId> = Some(BlockId(1075));
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
            id: "minecraft:tuff_slab",
            width: 3,
            height: 1,
            symmetrical: true,
            cells: &[
                Some(ItemSet::Direct(&[ItemId(12)])),
                Some(ItemSet::Direct(&[ItemId(12)])),
                Some(ItemSet::Direct(&[ItemId(12)])),
            ],
            product: Product::of(6),
        },
        Craft::Stonecutting {
            id: "minecraft:tuff_slab_from_tuff_stonecutting",
            ingredient: ItemSet::Direct(&[ItemId(12)]),
            count: 2,
        },
    ];
    type Behavior = crate::family::BlockItem;
}
