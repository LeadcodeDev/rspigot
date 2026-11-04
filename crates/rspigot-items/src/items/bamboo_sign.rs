use rspigot_blocks::BlockId;

use crate::{
    AttributeModifier, Craft, EquipmentSlot, Food, Fuel, Item, ItemId, ItemSet, Product, Rarity,
    Tool,
};

pub struct BambooSign;

impl Item for BambooSign {
    const NAME: &'static str = "minecraft:bamboo_sign";
    const ID: ItemId = ItemId(1112);
    const CLASS: &'static str = "StandingAndWallBlockItem";
    const MAX_STACK_SIZE: u8 = 16;
    const MAX_DAMAGE: Option<u32> = None;
    const WEAPON_DAMAGE: Option<u32> = None;
    const RARITY: Rarity = Rarity::Common;
    const BLOCK: Option<BlockId> = Some(BlockId(264));
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
        id: "minecraft:bamboo_sign",
        width: 3,
        height: 3,
        symmetrical: true,
        cells: &[
            Some(ItemSet::Direct(&[ItemId(73)])),
            Some(ItemSet::Direct(&[ItemId(73)])),
            Some(ItemSet::Direct(&[ItemId(73)])),
            Some(ItemSet::Direct(&[ItemId(73)])),
            Some(ItemSet::Direct(&[ItemId(73)])),
            Some(ItemSet::Direct(&[ItemId(73)])),
            None,
            Some(ItemSet::Direct(&[ItemId(1060)])),
            None,
        ],
        product: Product::of(3),
    }];
    type Behavior = crate::family::BlockItem;
}
