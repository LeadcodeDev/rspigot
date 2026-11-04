use rspigot_blocks::BlockId;

use crate::{
    AttributeModifier, Craft, EquipmentSlot, Food, Item, ItemId, ItemSet, Product, Rarity, Tool,
};

pub struct WritableBook;

impl Item for WritableBook {
    const NAME: &'static str = "minecraft:writable_book";
    const ID: ItemId = ItemId(1371);
    const CLASS: &'static str = "WritableBookItem";
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
    const CRAFTS: &'static [Craft] = &[Craft::Shapeless {
        id: "minecraft:writable_book",
        ingredients: &[
            ItemSet::Direct(&[ItemId(1146)]),
            ItemSet::Direct(&[ItemId(1196)]),
            ItemSet::Direct(&[ItemId(1063)]),
        ],
        product: Product::of(1),
    }];
    type Behavior = crate::family::Plain;
}
