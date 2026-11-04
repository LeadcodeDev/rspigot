use rspigot_blocks::BlockId;

use crate::{
    AttributeModifier, Craft, EquipmentSlot, Food, Item, ItemId, ItemSet, Product, Rarity, Tool,
};

pub struct WaxedWeatheredCopperChest;

impl Item for WaxedWeatheredCopperChest {
    const NAME: &'static str = "minecraft:waxed_weathered_copper_chest";
    const ID: ItemId = ItemId(1643);
    const CLASS: &'static str = "BlockItem";
    const MAX_STACK_SIZE: u8 = 64;
    const MAX_DAMAGE: Option<u32> = None;
    const WEAPON_DAMAGE: Option<u32> = None;
    const RARITY: Rarity = Rarity::Common;
    const BLOCK: Option<BlockId> = Some(BlockId(1204));
    const TOOL: Option<Tool> = None;
    const FOOD: Option<Food> = None;
    const EQUIPPABLE: Option<EquipmentSlot> = None;
    const ENCHANTABLE: Option<u32> = None;
    const REPAIRABLE: Option<ItemSet> = None;
    const DAMAGE_RESISTANT: Option<&'static str> = None;
    const USE_REMAINDER: Option<ItemId> = None;
    const ATTRIBUTE_MODIFIERS: &'static [AttributeModifier] = &[];
    const CRAFTS: &'static [Craft] = &[Craft::Shapeless {
        id: "minecraft:waxed_weathered_copper_chest_from_honeycomb",
        ingredients: &[ItemSet::Direct(&[ItemId(1639)]), ItemSet::Direct(&[ItemId(1530)])],
        product: Product::of(1),
    }];
    type Behavior = crate::family::BlockItem;
}
