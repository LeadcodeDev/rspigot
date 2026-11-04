use rspigot_blocks::BlockId;

use crate::{
    AttributeModifier, Craft, EquipmentSlot, Food, Fuel, Item, ItemId, ItemSet, Product, Rarity,
    Tool,
};

pub struct BirchBoat;

impl Item for BirchBoat {
    const NAME: &'static str = "minecraft:birch_boat";
    const ID: ItemId = ItemId(979);
    const CLASS: &'static str = "BoatItem";
    const MAX_STACK_SIZE: u8 = 1;
    const MAX_DAMAGE: Option<u32> = None;
    const WEAPON_DAMAGE: Option<u32> = None;
    const RARITY: Rarity = Rarity::Common;
    const BLOCK: Option<BlockId> = None;
    const TOOL: Option<Tool> = None;
    const FOOD: Option<Food> = None;
    const EQUIPPABLE: Option<EquipmentSlot> = None;
    const ENCHANTABLE: Option<u32> = None;
    const REPAIRABLE: Option<ItemSet> = None;
    const DAMAGE_RESISTANT: Option<&'static str> = None;
    const USE_REMAINDER: Option<ItemId> = None;
    const ATTRIBUTE_MODIFIERS: &'static [AttributeModifier] = &[];
    const COOKING_FUEL: Option<Fuel> = Some(Fuel { burn_time: 1200, fast_burn_time: 600 });
    const CRAFTS: &'static [Craft] = &[Craft::Shaped {
        id: "minecraft:birch_boat",
        width: 3,
        height: 2,
        symmetrical: true,
        cells: &[
            Some(ItemSet::Direct(&[ItemId(65)])),
            None,
            Some(ItemSet::Direct(&[ItemId(65)])),
            Some(ItemSet::Direct(&[ItemId(65)])),
            Some(ItemSet::Direct(&[ItemId(65)])),
            Some(ItemSet::Direct(&[ItemId(65)])),
        ],
        product: Product::of(1),
    }];
    type Behavior = crate::family::Plain;
}
