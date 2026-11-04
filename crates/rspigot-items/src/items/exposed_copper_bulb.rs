use rspigot_blocks::BlockId;

use crate::{
    AttributeModifier, Craft, EquipmentSlot, Food, Item, ItemId, ItemSet, Product, Rarity, Tool,
};

pub struct ExposedCopperBulb;

impl Item for ExposedCopperBulb {
    const NAME: &'static str = "minecraft:exposed_copper_bulb";
    const ID: ItemId = ItemId(1630);
    const CLASS: &'static str = "BlockItem";
    const MAX_STACK_SIZE: u8 = 64;
    const MAX_DAMAGE: Option<u32> = None;
    const WEAPON_DAMAGE: Option<u32> = None;
    const RARITY: Rarity = Rarity::Common;
    const BLOCK: Option<BlockId> = Some(BlockId(1191));
    const TOOL: Option<Tool> = None;
    const FOOD: Option<Food> = None;
    const EQUIPPABLE: Option<EquipmentSlot> = None;
    const ENCHANTABLE: Option<u32> = None;
    const REPAIRABLE: Option<ItemSet> = None;
    const DAMAGE_RESISTANT: Option<&'static str> = None;
    const USE_REMAINDER: Option<ItemId> = None;
    const ATTRIBUTE_MODIFIERS: &'static [AttributeModifier] = &[];
    const CRAFTS: &'static [Craft] = &[Craft::Shaped {
        id: "minecraft:exposed_copper_bulb",
        width: 3,
        height: 3,
        symmetrical: true,
        cells: &[
            None,
            Some(ItemSet::Direct(&[ItemId(121)])),
            None,
            Some(ItemSet::Direct(&[ItemId(121)])),
            Some(ItemSet::Direct(&[ItemId(1266)])),
            Some(ItemSet::Direct(&[ItemId(121)])),
            None,
            Some(ItemSet::Direct(&[ItemId(824)])),
            None,
        ],
        product: Product::of(4),
    }];
    type Behavior = crate::family::BlockItem;
}
