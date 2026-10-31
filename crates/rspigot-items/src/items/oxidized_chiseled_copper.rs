use rspigot_blocks::BlockId;

use crate::{
    AttributeModifier, Craft, EquipmentSlot, Food, Item, ItemId, ItemSet, Product, Rarity, Tool,
};

pub struct OxidizedChiseledCopper;

impl Item for OxidizedChiseledCopper {
    const NAME: &'static str = "minecraft:oxidized_chiseled_copper";
    const ID: ItemId = ItemId(134);
    const CLASS: &'static str = "BlockItem";
    const MAX_STACK_SIZE: u8 = 64;
    const MAX_DAMAGE: Option<u32> = None;
    const WEAPON_DAMAGE: Option<u32> = None;
    const RARITY: Rarity = Rarity::Common;
    const BLOCK: Option<BlockId> = Some(BlockId(1145));
    const TOOL: Option<Tool> = None;
    const FOOD: Option<Food> = None;
    const EQUIPPABLE: Option<EquipmentSlot> = None;
    const ENCHANTABLE: Option<u32> = None;
    const REPAIRABLE: Option<ItemSet> = None;
    const DAMAGE_RESISTANT: Option<&'static str> = None;
    const USE_REMAINDER: Option<ItemId> = None;
    const ATTRIBUTE_MODIFIERS: &'static [AttributeModifier] = &[];
    const CRAFTS: &'static [Craft] = &[
        Craft::Shaped {
            width: 1,
            height: 2,
            symmetrical: true,
            cells: &[Some(ItemSet::Direct(&[ItemId(158)])), Some(ItemSet::Direct(&[ItemId(158)]))],
            product: Product::of(1),
        },
        Craft::Stonecutting { ingredient: ItemSet::Direct(&[ItemId(123)]), count: 4 },
        Craft::Stonecutting { ingredient: ItemSet::Direct(&[ItemId(142)]), count: 1 },
    ];
    type Behavior = crate::family::BlockItem;
}
