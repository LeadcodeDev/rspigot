use rspigot_blocks::BlockId;

use crate::{
    AttributeModifier, Craft, EquipmentSlot, Food, Fuel, Item, ItemId, ItemSet, Product, Rarity,
    Tool,
};

pub struct PaleOakTrapdoor;

impl Item for PaleOakTrapdoor {
    const NAME: &'static str = "minecraft:pale_oak_trapdoor";
    const ID: ItemId = ItemId(918);
    const CLASS: &'static str = "BlockItem";
    const MAX_STACK_SIZE: u8 = 64;
    const MAX_DAMAGE: Option<u32> = None;
    const WEAPON_DAMAGE: Option<u32> = None;
    const RARITY: Rarity = Rarity::Common;
    const BLOCK: Option<BlockId> = Some(BlockId(372));
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
        width: 3,
        height: 2,
        symmetrical: true,
        cells: &[
            Some(ItemSet::Direct(&[ItemId(70)])),
            Some(ItemSet::Direct(&[ItemId(70)])),
            Some(ItemSet::Direct(&[ItemId(70)])),
            Some(ItemSet::Direct(&[ItemId(70)])),
            Some(ItemSet::Direct(&[ItemId(70)])),
            Some(ItemSet::Direct(&[ItemId(70)])),
        ],
        product: Product::of(2),
    }];
    type Behavior = crate::family::BlockItem;
}
