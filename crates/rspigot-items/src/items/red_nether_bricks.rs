use rspigot_blocks::BlockId;

use crate::{
    AttributeModifier, Craft, EquipmentSlot, Food, Item, ItemId, ItemSet, Product, Rarity, Tool,
};

pub struct RedNetherBricks;

impl Item for RedNetherBricks {
    const NAME: &'static str = "minecraft:red_nether_bricks";
    const ID: ItemId = ItemId(653);
    const CLASS: &'static str = "BlockItem";
    const MAX_STACK_SIZE: u8 = 64;
    const MAX_DAMAGE: Option<u32> = None;
    const WEAPON_DAMAGE: Option<u32> = None;
    const RARITY: Rarity = Rarity::Common;
    const BLOCK: Option<BlockId> = Some(BlockId(731));
    const TOOL: Option<Tool> = None;
    const FOOD: Option<Food> = None;
    const EQUIPPABLE: Option<EquipmentSlot> = None;
    const ENCHANTABLE: Option<u32> = None;
    const REPAIRABLE: Option<ItemSet> = None;
    const DAMAGE_RESISTANT: Option<&'static str> = None;
    const USE_REMAINDER: Option<ItemId> = None;
    const ATTRIBUTE_MODIFIERS: &'static [AttributeModifier] = &[];
    const CRAFTS: &'static [Craft] = &[Craft::Shaped {
        id: "minecraft:red_nether_bricks",
        width: 2,
        height: 2,
        symmetrical: false,
        cells: &[
            Some(ItemSet::Direct(&[ItemId(1396)])),
            Some(ItemSet::Direct(&[ItemId(1269)])),
            Some(ItemSet::Direct(&[ItemId(1269)])),
            Some(ItemSet::Direct(&[ItemId(1396)])),
        ],
        product: Product::of(1),
    }];
    type Behavior = crate::family::BlockItem;
}
