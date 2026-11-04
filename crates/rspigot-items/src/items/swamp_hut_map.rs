use rspigot_blocks::BlockId;

use crate::{AttributeModifier, Craft, EquipmentSlot, Food, Item, ItemId, ItemSet, Rarity, Tool};

pub struct SwampHutMap;

impl Item for SwampHutMap {
    const NAME: &'static str = "minecraft:swamp_hut_map";
    const ID: ItemId = ItemId(1243);
    const CLASS: &'static str = "MapItem";
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
    const CRAFTS: &'static [Craft] = &[Craft::Transmute {
        id: "minecraft:map_cloning",
        input: ItemSet::Direct(&[ItemId(1243)]),
        material: ItemSet::Direct(&[ItemId(1382)]),
        min_material: 1,
        max_material: 8,
        add_material_count: true,
        count: 1,
    }];
    type Behavior = crate::family::Plain;
}
