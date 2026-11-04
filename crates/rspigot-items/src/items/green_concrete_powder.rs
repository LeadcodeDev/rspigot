use rspigot_blocks::BlockId;

use crate::{
    AttributeModifier, Craft, EquipmentSlot, Food, Item, ItemId, ItemSet, Product, Rarity, Tool,
};

pub struct GreenConcretePowder;

impl Item for GreenConcretePowder {
    const NAME: &'static str = "minecraft:green_concrete_powder";
    const ID: ItemId = ItemId(750);
    const CLASS: &'static str = "BlockItem";
    const MAX_STACK_SIZE: u8 = 64;
    const MAX_DAMAGE: Option<u32> = None;
    const WEAPON_DAMAGE: Option<u32> = None;
    const RARITY: Rarity = Rarity::Common;
    const BLOCK: Option<BlockId> = Some(BlockId(829));
    const TOOL: Option<Tool> = None;
    const FOOD: Option<Food> = None;
    const EQUIPPABLE: Option<EquipmentSlot> = None;
    const ENCHANTABLE: Option<u32> = None;
    const REPAIRABLE: Option<ItemSet> = None;
    const DAMAGE_RESISTANT: Option<&'static str> = None;
    const USE_REMAINDER: Option<ItemId> = None;
    const ATTRIBUTE_MODIFIERS: &'static [AttributeModifier] = &[];
    const CRAFTS: &'static [Craft] = &[Craft::Shapeless {
        id: "minecraft:green_concrete_powder",
        ingredients: &[
            ItemSet::Direct(&[ItemId(1212)]),
            ItemSet::Direct(&[ItemId(88)]),
            ItemSet::Direct(&[ItemId(88)]),
            ItemSet::Direct(&[ItemId(88)]),
            ItemSet::Direct(&[ItemId(88)]),
            ItemSet::Direct(&[ItemId(92)]),
            ItemSet::Direct(&[ItemId(92)]),
            ItemSet::Direct(&[ItemId(92)]),
            ItemSet::Direct(&[ItemId(92)]),
        ],
        product: Product::of(8),
    }];
    type Behavior = crate::family::BlockItem;
}
