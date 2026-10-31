use rspigot_blocks::BlockId;

use crate::{
    AttributeModifier, Craft, EquipmentSlot, Food, Item, ItemId, ItemSet, Product, Rarity, Tool,
};

pub struct GrayTerracotta;

impl Item for GrayTerracotta {
    const NAME: &'static str = "minecraft:gray_terracotta";
    const ID: ItemId = ItemId(568);
    const CLASS: &'static str = "BlockItem";
    const MAX_STACK_SIZE: u8 = 64;
    const MAX_DAMAGE: Option<u32> = None;
    const WEAPON_DAMAGE: Option<u32> = None;
    const RARITY: Rarity = Rarity::Common;
    const BLOCK: Option<BlockId> = Some(BlockId(544));
    const TOOL: Option<Tool> = None;
    const FOOD: Option<Food> = None;
    const EQUIPPABLE: Option<EquipmentSlot> = None;
    const ENCHANTABLE: Option<u32> = None;
    const REPAIRABLE: Option<ItemSet> = None;
    const DAMAGE_RESISTANT: Option<&'static str> = None;
    const USE_REMAINDER: Option<ItemId> = None;
    const ATTRIBUTE_MODIFIERS: &'static [AttributeModifier] = &[];
    const CRAFTS: &'static [Craft] = &[Craft::Shaped {
        width: 3,
        height: 3,
        symmetrical: true,
        cells: &[
            Some(ItemSet::Direct(&[ItemId(596)])),
            Some(ItemSet::Direct(&[ItemId(596)])),
            Some(ItemSet::Direct(&[ItemId(596)])),
            Some(ItemSet::Direct(&[ItemId(596)])),
            Some(ItemSet::Direct(&[ItemId(1206)])),
            Some(ItemSet::Direct(&[ItemId(596)])),
            Some(ItemSet::Direct(&[ItemId(596)])),
            Some(ItemSet::Direct(&[ItemId(596)])),
            Some(ItemSet::Direct(&[ItemId(596)])),
        ],
        product: Product::of(8),
    }];
    type Behavior = crate::family::BlockItem;
}
