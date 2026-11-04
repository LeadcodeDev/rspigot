use rspigot_blocks::BlockId;

use crate::{
    AttributeModifier, Craft, EquipmentSlot, Food, Item, ItemId, ItemSet, Product, Rarity, Tool,
};

pub struct ChiseledRedSandstone;

impl Item for ChiseledRedSandstone {
    const NAME: &'static str = "minecraft:chiseled_red_sandstone";
    const ID: ItemId = ItemId(645);
    const CLASS: &'static str = "BlockItem";
    const MAX_STACK_SIZE: u8 = 64;
    const MAX_DAMAGE: Option<u32> = None;
    const WEAPON_DAMAGE: Option<u32> = None;
    const RARITY: Rarity = Rarity::Common;
    const BLOCK: Option<BlockId> = Some(BlockId(650));
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
            id: "minecraft:chiseled_red_sandstone",
            width: 1,
            height: 2,
            symmetrical: true,
            cells: &[Some(ItemSet::Direct(&[ItemId(366)])), Some(ItemSet::Direct(&[ItemId(366)]))],
            product: Product::of(1),
        },
        Craft::Stonecutting {
            id: "minecraft:chiseled_red_sandstone_from_red_sandstone_stonecutting",
            ingredient: ItemSet::Direct(&[ItemId(644)]),
            count: 1,
        },
    ];
    type Behavior = crate::family::BlockItem;
}
