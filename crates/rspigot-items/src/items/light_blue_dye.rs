use rspigot_blocks::BlockId;

use crate::{
    AttributeModifier, Craft, EquipmentSlot, Food, Item, ItemId, ItemSet, Product, Rarity, Tool,
};

pub struct LightBlueDye;

impl Item for LightBlueDye {
    const NAME: &'static str = "minecraft:light_blue_dye";
    const ID: ItemId = ItemId(1202);
    const CLASS: &'static str = "DyeItem";
    const MAX_STACK_SIZE: u8 = 64;
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
    const CRAFTS: &'static [Craft] = &[
        Craft::Shapeless {
            ingredients: &[ItemSet::Direct(&[ItemId(303)])],
            product: Product::of(1),
        },
        Craft::Shapeless {
            ingredients: &[ItemSet::Direct(&[ItemId(1210)]), ItemSet::Direct(&[ItemId(1199)])],
            product: Product::of(2),
        },
    ];
    type Behavior = crate::family::Plain;
}
