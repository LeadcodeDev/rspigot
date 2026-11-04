use rspigot_blocks::BlockId;

use crate::{
    AttributeModifier, Craft, EquipmentSlot, Food, Item, ItemId, ItemSet, Rarity, Tool, tags,
};

pub struct GrayBundle;

impl Item for GrayBundle {
    const NAME: &'static str = "minecraft:gray_bundle";
    const ID: ItemId = ItemId(1161);
    const CLASS: &'static str = "BundleItem";
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
    const CRAFTS: &'static [Craft] = &[Craft::Transmute {
        id: "minecraft:gray_bundle",
        input: ItemSet::Tag(&tags::ITEM_BUNDLES),
        material: ItemSet::Direct(&[ItemId(1206)]),
        min_material: 1,
        max_material: 1,
        add_material_count: false,
        count: 1,
    }];
    type Behavior = crate::family::Plain;
}
