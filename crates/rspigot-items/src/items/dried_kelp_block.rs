use rspigot_blocks::BlockId;

use crate::{
    AttributeModifier, Craft, EquipmentSlot, Food, Fuel, Item, ItemId, ItemSet, Product, Rarity,
    Tool,
};

pub struct DriedKelpBlock;

impl Item for DriedKelpBlock {
    const NAME: &'static str = "minecraft:dried_kelp_block";
    const ID: ItemId = ItemId(1144);
    const CLASS: &'static str = "BlockItem";
    const MAX_STACK_SIZE: u8 = 64;
    const MAX_DAMAGE: Option<u32> = None;
    const WEAPON_DAMAGE: Option<u32> = None;
    const RARITY: Rarity = Rarity::Common;
    const BLOCK: Option<BlockId> = Some(BlockId(834));
    const TOOL: Option<Tool> = None;
    const FOOD: Option<Food> = None;
    const EQUIPPABLE: Option<EquipmentSlot> = None;
    const ENCHANTABLE: Option<u32> = None;
    const REPAIRABLE: Option<ItemSet> = None;
    const DAMAGE_RESISTANT: Option<&'static str> = None;
    const USE_REMAINDER: Option<ItemId> = None;
    const ATTRIBUTE_MODIFIERS: &'static [AttributeModifier] = &[];
    const COOKING_FUEL: Option<Fuel> = Some(Fuel { burn_time: 4001, fast_burn_time: 2000 });
    const CRAFTS: &'static [Craft] = &[Craft::Shaped {
        id: "minecraft:dried_kelp_block",
        width: 3,
        height: 3,
        symmetrical: true,
        cells: &[
            Some(ItemSet::Direct(&[ItemId(1257)])),
            Some(ItemSet::Direct(&[ItemId(1257)])),
            Some(ItemSet::Direct(&[ItemId(1257)])),
            Some(ItemSet::Direct(&[ItemId(1257)])),
            Some(ItemSet::Direct(&[ItemId(1257)])),
            Some(ItemSet::Direct(&[ItemId(1257)])),
            Some(ItemSet::Direct(&[ItemId(1257)])),
            Some(ItemSet::Direct(&[ItemId(1257)])),
            Some(ItemSet::Direct(&[ItemId(1257)])),
        ],
        product: Product::of(1),
    }];
    type Behavior = crate::family::BlockItem;
}
