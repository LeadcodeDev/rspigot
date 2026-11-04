use rspigot_blocks::BlockId;

use crate::{
    AttributeModifier, Craft, EquipmentSlot, Food, Fuel, Item, ItemId, ItemSet, Product, Rarity,
    Tool,
};

pub struct AcaciaFenceGate;

impl Item for AcaciaFenceGate {
    const NAME: &'static str = "minecraft:acacia_fence_gate";
    const ID: ItemId = ItemId(936);
    const CLASS: &'static str = "BlockItem";
    const MAX_STACK_SIZE: u8 = 64;
    const MAX_DAMAGE: Option<u32> = None;
    const WEAPON_DAMAGE: Option<u32> = None;
    const RARITY: Rarity = Rarity::Common;
    const BLOCK: Option<BlockId> = Some(BlockId(685));
    const TOOL: Option<Tool> = None;
    const FOOD: Option<Food> = None;
    const EQUIPPABLE: Option<EquipmentSlot> = None;
    const ENCHANTABLE: Option<u32> = None;
    const REPAIRABLE: Option<ItemSet> = None;
    const DAMAGE_RESISTANT: Option<&'static str> = None;
    const USE_REMAINDER: Option<ItemId> = None;
    const ATTRIBUTE_MODIFIERS: &'static [AttributeModifier] = &[];
    const COOKING_FUEL: Option<Fuel> = Some(Fuel { burn_time: 300, fast_burn_time: 150 });
    const CRAFTS: &'static [Craft] = &[Craft::Shaped {
        id: "minecraft:acacia_fence_gate",
        width: 3,
        height: 2,
        symmetrical: true,
        cells: &[
            Some(ItemSet::Direct(&[ItemId(1060)])),
            Some(ItemSet::Direct(&[ItemId(67)])),
            Some(ItemSet::Direct(&[ItemId(1060)])),
            Some(ItemSet::Direct(&[ItemId(1060)])),
            Some(ItemSet::Direct(&[ItemId(67)])),
            Some(ItemSet::Direct(&[ItemId(1060)])),
        ],
        product: Product::of(1),
    }];
    type Behavior = crate::family::BlockItem;
}
