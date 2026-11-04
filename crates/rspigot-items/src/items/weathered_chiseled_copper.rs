use rspigot_blocks::BlockId;

use crate::{
    AttributeModifier, Craft, EquipmentSlot, Food, Item, ItemId, ItemSet, Product, Rarity, Tool,
};

pub struct WeatheredChiseledCopper;

impl Item for WeatheredChiseledCopper {
    const NAME: &'static str = "minecraft:weathered_chiseled_copper";
    const ID: ItemId = ItemId(133);
    const CLASS: &'static str = "BlockItem";
    const MAX_STACK_SIZE: u8 = 64;
    const MAX_DAMAGE: Option<u32> = None;
    const WEAPON_DAMAGE: Option<u32> = None;
    const RARITY: Rarity = Rarity::Common;
    const BLOCK: Option<BlockId> = Some(BlockId(1144));
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
            id: "minecraft:weathered_chiseled_copper",
            width: 1,
            height: 2,
            symmetrical: true,
            cells: &[Some(ItemSet::Direct(&[ItemId(157)])), Some(ItemSet::Direct(&[ItemId(157)]))],
            product: Product::of(1),
        },
        Craft::Stonecutting {
            id: "minecraft:weathered_chiseled_copper_from_weathered_copper_stonecutting",
            ingredient: ItemSet::Direct(&[ItemId(122)]),
            count: 4,
        },
        Craft::Stonecutting {
            id: "minecraft:weathered_chiseled_copper_from_weathered_cut_copper_stonecutting",
            ingredient: ItemSet::Direct(&[ItemId(141)]),
            count: 1,
        },
    ];
    type Behavior = crate::family::BlockItem;
}
