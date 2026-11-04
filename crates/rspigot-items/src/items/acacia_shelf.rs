use rspigot_blocks::BlockId;

use crate::{
    AttributeModifier, Craft, EquipmentSlot, Food, Fuel, Item, ItemId, ItemSet, Product, Rarity,
    Tool,
};

pub struct AcaciaShelf;

impl Item for AcaciaShelf {
    const NAME: &'static str = "minecraft:acacia_shelf";
    const ID: ItemId = ItemId(377);
    const CLASS: &'static str = "BlockItem";
    const MAX_STACK_SIZE: u8 = 64;
    const MAX_DAMAGE: Option<u32> = None;
    const WEAPON_DAMAGE: Option<u32> = None;
    const RARITY: Rarity = Rarity::Common;
    const BLOCK: Option<BlockId> = Some(BlockId(223));
    const TOOL: Option<Tool> = None;
    const FOOD: Option<Food> = None;
    const EQUIPPABLE: Option<EquipmentSlot> = None;
    const ENCHANTABLE: Option<u32> = None;
    const REPAIRABLE: Option<ItemSet> = None;
    const DAMAGE_RESISTANT: Option<&'static str> = None;
    const USE_REMAINDER: Option<ItemId> = None;
    const ATTRIBUTE_MODIFIERS: &'static [AttributeModifier] = &[];
    const COOKING_FUEL: Option<Fuel> = Some(Fuel { burn_time: 300, fast_burn_time: 150 });
    const CRAFTS: &'static [Craft] = &[Craft::Shaped {
        id: "minecraft:acacia_shelf",
        width: 3,
        height: 3,
        symmetrical: true,
        cells: &[
            Some(ItemSet::Direct(&[ItemId(182)])),
            Some(ItemSet::Direct(&[ItemId(182)])),
            Some(ItemSet::Direct(&[ItemId(182)])),
            None,
            None,
            None,
            Some(ItemSet::Direct(&[ItemId(182)])),
            Some(ItemSet::Direct(&[ItemId(182)])),
            Some(ItemSet::Direct(&[ItemId(182)])),
        ],
        product: Product::of(6),
    }];
    type Behavior = crate::family::BlockItem;
}
