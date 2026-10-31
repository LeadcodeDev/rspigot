use rspigot_blocks::BlockId;

use crate::{
    AttributeModifier, Craft, EquipmentSlot, Food, Item, ItemId, ItemSet, Product, Rarity, Tool,
};

pub struct WarpedFungusOnAStick;

impl Item for WarpedFungusOnAStick {
    const NAME: &'static str = "minecraft:warped_fungus_on_a_stick";
    const ID: ItemId = ItemId(972);
    const CLASS: &'static str = "FoodOnAStickItem";
    const MAX_STACK_SIZE: u8 = 1;
    const MAX_DAMAGE: Option<u32> = Some(100);
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
    const CRAFTS: &'static [Craft] = &[Craft::Shaped {
        width: 2,
        height: 2,
        symmetrical: false,
        cells: &[
            Some(ItemSet::Direct(&[ItemId(1186)])),
            None,
            None,
            Some(ItemSet::Direct(&[ItemId(321)])),
        ],
        product: Product::of(1),
    }];
    type Behavior = crate::family::Plain;
}
