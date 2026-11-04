use rspigot_blocks::BlockId;

use crate::{
    AttributeModifier, Craft, EquipmentSlot, Food, Fuel, Item, ItemId, ItemSet, Product, Rarity,
    Tool,
};

pub struct CoalBlock;

impl Item for CoalBlock {
    const NAME: &'static str = "minecraft:coal_block";
    const ID: ItemId = ItemId(112);
    const CLASS: &'static str = "BlockItem";
    const MAX_STACK_SIZE: u8 = 64;
    const MAX_DAMAGE: Option<u32> = None;
    const WEAPON_DAMAGE: Option<u32> = None;
    const RARITY: Rarity = Rarity::Common;
    const BLOCK: Option<BlockId> = Some(BlockId(609));
    const TOOL: Option<Tool> = None;
    const FOOD: Option<Food> = None;
    const EQUIPPABLE: Option<EquipmentSlot> = None;
    const ENCHANTABLE: Option<u32> = None;
    const REPAIRABLE: Option<ItemSet> = None;
    const DAMAGE_RESISTANT: Option<&'static str> = None;
    const USE_REMAINDER: Option<ItemId> = None;
    const ATTRIBUTE_MODIFIERS: &'static [AttributeModifier] = &[];
    const COOKING_FUEL: Option<Fuel> = Some(Fuel { burn_time: 16000, fast_burn_time: 8000 });
    const CRAFTS: &'static [Craft] = &[Craft::Shaped {
        id: "minecraft:coal_block",
        width: 3,
        height: 3,
        symmetrical: true,
        cells: &[
            Some(ItemSet::Direct(&[ItemId(1010)])),
            Some(ItemSet::Direct(&[ItemId(1010)])),
            Some(ItemSet::Direct(&[ItemId(1010)])),
            Some(ItemSet::Direct(&[ItemId(1010)])),
            Some(ItemSet::Direct(&[ItemId(1010)])),
            Some(ItemSet::Direct(&[ItemId(1010)])),
            Some(ItemSet::Direct(&[ItemId(1010)])),
            Some(ItemSet::Direct(&[ItemId(1010)])),
            Some(ItemSet::Direct(&[ItemId(1010)])),
        ],
        product: Product::of(1),
    }];
    type Behavior = crate::family::BlockItem;
}
