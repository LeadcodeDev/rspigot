use rspigot_blocks::BlockId;

use crate::{
    AttributeModifier, CookingKind, Craft, EquipmentSlot, Food, Item, ItemId, ItemSet, Rarity, Tool,
};

pub struct GrayGlazedTerracotta;

impl Item for GrayGlazedTerracotta {
    const NAME: &'static str = "minecraft:gray_glazed_terracotta";
    const ID: ItemId = ItemId(680);
    const CLASS: &'static str = "BlockItem";
    const MAX_STACK_SIZE: u8 = 64;
    const MAX_DAMAGE: Option<u32> = None;
    const WEAPON_DAMAGE: Option<u32> = None;
    const RARITY: Rarity = Rarity::Common;
    const BLOCK: Option<BlockId> = Some(BlockId(759));
    const TOOL: Option<Tool> = None;
    const FOOD: Option<Food> = None;
    const EQUIPPABLE: Option<EquipmentSlot> = None;
    const ENCHANTABLE: Option<u32> = None;
    const REPAIRABLE: Option<ItemSet> = None;
    const DAMAGE_RESISTANT: Option<&'static str> = None;
    const USE_REMAINDER: Option<ItemId> = None;
    const ATTRIBUTE_MODIFIERS: &'static [AttributeModifier] = &[];
    const CRAFTS: &'static [Craft] = &[Craft::Cooking {
        id: "minecraft:gray_glazed_terracotta",
        kind: CookingKind::Smelting,
        ingredient: ItemSet::Direct(&[ItemId(568)]),
        experience: 0.1,
        cooking_time: 200,
    }];
    type Behavior = crate::family::BlockItem;
}
