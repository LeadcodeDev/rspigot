use rspigot_blocks::BlockId;

use crate::{
    AttributeModifier, Craft, EquipmentSlot, Food, Item, ItemId, ItemSet, Product, Rarity, Tool,
};

pub struct RedSandstoneStairs;

impl Item for RedSandstoneStairs {
    const NAME: &'static str = "minecraft:red_sandstone_stairs";
    const ID: ItemId = ItemId(647);
    const CLASS: &'static str = "BlockItem";
    const MAX_STACK_SIZE: u8 = 64;
    const MAX_DAMAGE: Option<u32> = None;
    const WEAPON_DAMAGE: Option<u32> = None;
    const RARITY: Rarity = Rarity::Common;
    const BLOCK: Option<BlockId> = Some(BlockId(652));
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
        symmetrical: false,
        cells: &[
            Some(ItemSet::Direct(&[ItemId(644), ItemId(645), ItemId(646)])),
            None,
            None,
            Some(ItemSet::Direct(&[ItemId(644), ItemId(645), ItemId(646)])),
            Some(ItemSet::Direct(&[ItemId(644), ItemId(645), ItemId(646)])),
            None,
            Some(ItemSet::Direct(&[ItemId(644), ItemId(645), ItemId(646)])),
            Some(ItemSet::Direct(&[ItemId(644), ItemId(645), ItemId(646)])),
            Some(ItemSet::Direct(&[ItemId(644), ItemId(645), ItemId(646)])),
        ],
        product: Product::of(4),
    }];
    type Behavior = crate::family::BlockItem;
}
