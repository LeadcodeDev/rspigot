use rspigot_blocks::BlockId;

use crate::{AttributeModifier, Craft, EquipmentSlot, Food, Item, ItemId, ItemSet, Rarity, Tool};

pub struct Elytra;

impl Item for Elytra {
    const NAME: &'static str = "minecraft:elytra";
    const ID: ItemId = ItemId(974);
    const CLASS: &'static str = "Item";
    const MAX_STACK_SIZE: u8 = 1;
    const MAX_DAMAGE: Option<u32> = Some(432);
    const WEAPON_DAMAGE: Option<u32> = None;
    const RARITY: Rarity = Rarity::Epic;
    const BLOCK: Option<BlockId> = None;
    const TOOL: Option<Tool> = None;
    const FOOD: Option<Food> = None;
    const EQUIPPABLE: Option<EquipmentSlot> = Some(EquipmentSlot::Chest);
    const ENCHANTABLE: Option<u32> = None;
    const REPAIRABLE: Option<ItemSet> = Some(ItemSet::Direct(&[ItemId(973)]));
    const DAMAGE_RESISTANT: Option<&'static str> = None;
    const USE_REMAINDER: Option<ItemId> = None;
    const ATTRIBUTE_MODIFIERS: &'static [AttributeModifier] = &[];
    const CRAFTS: &'static [Craft] = &[Craft::RepairItem];
    type Behavior = crate::family::Plain;
}
