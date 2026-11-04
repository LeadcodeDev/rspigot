use rspigot_blocks::BlockId;

use crate::{
    AttributeModifier, Craft, EquipmentSlot, Food, Item, ItemId, ItemSet, Product, Rarity, Tool,
};

pub struct WaxedOxidizedCopperDoor;

impl Item for WaxedOxidizedCopperDoor {
    const NAME: &'static str = "minecraft:waxed_oxidized_copper_door";
    const ID: ItemId = ItemId(909);
    const CLASS: &'static str = "DoubleHighBlockItem";
    const MAX_STACK_SIZE: u8 = 64;
    const MAX_DAMAGE: Option<u32> = None;
    const WEAPON_DAMAGE: Option<u32> = None;
    const RARITY: Rarity = Rarity::Common;
    const BLOCK: Option<BlockId> = Some(BlockId(1173));
    const TOOL: Option<Tool> = None;
    const FOOD: Option<Food> = None;
    const EQUIPPABLE: Option<EquipmentSlot> = None;
    const ENCHANTABLE: Option<u32> = None;
    const REPAIRABLE: Option<ItemSet> = None;
    const DAMAGE_RESISTANT: Option<&'static str> = None;
    const USE_REMAINDER: Option<ItemId> = None;
    const ATTRIBUTE_MODIFIERS: &'static [AttributeModifier] = &[];
    const CRAFTS: &'static [Craft] = &[Craft::Shapeless {
        id: "minecraft:waxed_oxidized_copper_door_from_honeycomb",
        ingredients: &[ItemSet::Direct(&[ItemId(905)]), ItemSet::Direct(&[ItemId(1530)])],
        product: Product::of(1),
    }];
    type Behavior = crate::family::BlockItem;
}
