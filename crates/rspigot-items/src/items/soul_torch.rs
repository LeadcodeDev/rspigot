use rspigot_blocks::BlockId;

use crate::{
    AttributeModifier, Craft, EquipmentSlot, Food, Item, ItemId, ItemSet, Product, Rarity, Tool,
    tags,
};

pub struct SoulTorch;

impl Item for SoulTorch {
    const NAME: &'static str = "minecraft:soul_torch";
    const ID: ItemId = ItemId(439);
    const CLASS: &'static str = "StandingAndWallBlockItem";
    const MAX_STACK_SIZE: u8 = 64;
    const MAX_DAMAGE: Option<u32> = None;
    const WEAPON_DAMAGE: Option<u32> = None;
    const RARITY: Rarity = Rarity::Common;
    const BLOCK: Option<BlockId> = Some(BlockId(339));
    const TOOL: Option<Tool> = None;
    const FOOD: Option<Food> = None;
    const EQUIPPABLE: Option<EquipmentSlot> = None;
    const ENCHANTABLE: Option<u32> = None;
    const REPAIRABLE: Option<ItemSet> = None;
    const DAMAGE_RESISTANT: Option<&'static str> = None;
    const USE_REMAINDER: Option<ItemId> = None;
    const ATTRIBUTE_MODIFIERS: &'static [AttributeModifier] = &[];
    const CRAFTS: &'static [Craft] = &[Craft::Shaped {
        width: 1,
        height: 3,
        symmetrical: true,
        cells: &[
            Some(ItemSet::Direct(&[ItemId(1010), ItemId(1011)])),
            Some(ItemSet::Direct(&[ItemId(1060)])),
            Some(ItemSet::Tag(&tags::ITEM_SOUL_FIRE_BASE_BLOCKS)),
        ],
        product: Product::of(4),
    }];
    type Behavior = crate::family::BlockItem;
}
