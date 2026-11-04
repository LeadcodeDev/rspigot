use rspigot_blocks::BlockId;

use crate::{
    AttributeModifier, Craft, EquipmentSlot, Food, Fuel, Item, ItemId, ItemSet, Product, Rarity,
    Tool,
};

pub struct JungleDoor;

impl Item for JungleDoor {
    const NAME: &'static str = "minecraft:jungle_door";
    const ID: ItemId = ItemId(892);
    const CLASS: &'static str = "DoubleHighBlockItem";
    const MAX_STACK_SIZE: u8 = 64;
    const MAX_DAMAGE: Option<u32> = None;
    const WEAPON_DAMAGE: Option<u32> = None;
    const RARITY: Rarity = Rarity::Common;
    const BLOCK: Option<BlockId> = Some(BlockId(704));
    const TOOL: Option<Tool> = None;
    const FOOD: Option<Food> = None;
    const EQUIPPABLE: Option<EquipmentSlot> = None;
    const ENCHANTABLE: Option<u32> = None;
    const REPAIRABLE: Option<ItemSet> = None;
    const DAMAGE_RESISTANT: Option<&'static str> = None;
    const USE_REMAINDER: Option<ItemId> = None;
    const ATTRIBUTE_MODIFIERS: &'static [AttributeModifier] = &[];
    const COOKING_FUEL: Option<Fuel> = Some(Fuel { burn_time: 200, fast_burn_time: 100 });
    const CRAFTS: &'static [Craft] = &[Craft::Shaped {
        id: "minecraft:jungle_door",
        width: 2,
        height: 3,
        symmetrical: true,
        cells: &[
            Some(ItemSet::Direct(&[ItemId(66)])),
            Some(ItemSet::Direct(&[ItemId(66)])),
            Some(ItemSet::Direct(&[ItemId(66)])),
            Some(ItemSet::Direct(&[ItemId(66)])),
            Some(ItemSet::Direct(&[ItemId(66)])),
            Some(ItemSet::Direct(&[ItemId(66)])),
        ],
        product: Product::of(3),
    }];
    type Behavior = crate::family::BlockItem;
}
