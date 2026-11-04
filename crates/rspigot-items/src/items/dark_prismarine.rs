use rspigot_blocks::BlockId;

use crate::{
    AttributeModifier, Craft, EquipmentSlot, Food, Item, ItemId, ItemSet, Product, Rarity, Tool,
};

pub struct DarkPrismarine;

impl Item for DarkPrismarine {
    const NAME: &'static str = "minecraft:dark_prismarine";
    const ID: ItemId = ItemId(639);
    const CLASS: &'static str = "BlockItem";
    const MAX_STACK_SIZE: u8 = 64;
    const MAX_DAMAGE: Option<u32> = None;
    const WEAPON_DAMAGE: Option<u32> = None;
    const RARITY: Rarity = Rarity::Common;
    const BLOCK: Option<BlockId> = Some(BlockId(583));
    const TOOL: Option<Tool> = None;
    const FOOD: Option<Food> = None;
    const EQUIPPABLE: Option<EquipmentSlot> = None;
    const ENCHANTABLE: Option<u32> = None;
    const REPAIRABLE: Option<ItemSet> = None;
    const DAMAGE_RESISTANT: Option<&'static str> = None;
    const USE_REMAINDER: Option<ItemId> = None;
    const ATTRIBUTE_MODIFIERS: &'static [AttributeModifier] = &[];
    const CRAFTS: &'static [Craft] = &[Craft::Shaped {
        id: "minecraft:dark_prismarine",
        width: 3,
        height: 3,
        symmetrical: true,
        cells: &[
            Some(ItemSet::Direct(&[ItemId(1398)])),
            Some(ItemSet::Direct(&[ItemId(1398)])),
            Some(ItemSet::Direct(&[ItemId(1398)])),
            Some(ItemSet::Direct(&[ItemId(1398)])),
            Some(ItemSet::Direct(&[ItemId(1214)])),
            Some(ItemSet::Direct(&[ItemId(1398)])),
            Some(ItemSet::Direct(&[ItemId(1398)])),
            Some(ItemSet::Direct(&[ItemId(1398)])),
            Some(ItemSet::Direct(&[ItemId(1398)])),
        ],
        product: Product::of(1),
    }];
    type Behavior = crate::family::BlockItem;
}
