use rspigot_blocks::BlockId;

use crate::{
    AttributeModifier, Craft, EquipmentSlot, Food, Item, ItemId, ItemSet, Product, Rarity, Tool,
};

pub struct BlueStainedGlass;

impl Item for BlueStainedGlass {
    const NAME: &'static str = "minecraft:blue_stained_glass";
    const ID: ItemId = ItemId(616);
    const CLASS: &'static str = "BlockItem";
    const MAX_STACK_SIZE: u8 = 64;
    const MAX_DAMAGE: Option<u32> = None;
    const WEAPON_DAMAGE: Option<u32> = None;
    const RARITY: Rarity = Rarity::Common;
    const BLOCK: Option<BlockId> = Some(BlockId(360));
    const TOOL: Option<Tool> = None;
    const FOOD: Option<Food> = None;
    const EQUIPPABLE: Option<EquipmentSlot> = None;
    const ENCHANTABLE: Option<u32> = None;
    const REPAIRABLE: Option<ItemSet> = None;
    const DAMAGE_RESISTANT: Option<&'static str> = None;
    const USE_REMAINDER: Option<ItemId> = None;
    const ATTRIBUTE_MODIFIERS: &'static [AttributeModifier] = &[];
    const CRAFTS: &'static [Craft] = &[Craft::Shaped {
        id: "minecraft:blue_stained_glass",
        width: 3,
        height: 3,
        symmetrical: true,
        cells: &[
            Some(ItemSet::Direct(&[ItemId(231)])),
            Some(ItemSet::Direct(&[ItemId(231)])),
            Some(ItemSet::Direct(&[ItemId(231)])),
            Some(ItemSet::Direct(&[ItemId(231)])),
            Some(ItemSet::Direct(&[ItemId(1210)])),
            Some(ItemSet::Direct(&[ItemId(231)])),
            Some(ItemSet::Direct(&[ItemId(231)])),
            Some(ItemSet::Direct(&[ItemId(231)])),
            Some(ItemSet::Direct(&[ItemId(231)])),
        ],
        product: Product::of(8),
    }];
    type Behavior = crate::family::BlockItem;
}
