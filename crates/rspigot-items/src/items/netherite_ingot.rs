use rspigot_blocks::BlockId;

use crate::{
    AttributeModifier, Craft, EquipmentSlot, Food, Item, ItemId, ItemSet, Product, Rarity, Tool,
};

pub struct NetheriteIngot;

impl Item for NetheriteIngot {
    const NAME: &'static str = "minecraft:netherite_ingot";
    const ID: ItemId = ItemId(1023);
    const CLASS: &'static str = "Item";
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
    const DAMAGE_RESISTANT: Option<&'static str> = Some("minecraft:is_fire");
    const USE_REMAINDER: Option<ItemId> = None;
    const ATTRIBUTE_MODIFIERS: &'static [AttributeModifier] = &[];
    const CRAFTS: &'static [Craft] = &[
        Craft::Shapeless {
            ingredients: &[
                ItemSet::Direct(&[ItemId(1024)]),
                ItemSet::Direct(&[ItemId(1024)]),
                ItemSet::Direct(&[ItemId(1024)]),
                ItemSet::Direct(&[ItemId(1024)]),
                ItemSet::Direct(&[ItemId(1022)]),
                ItemSet::Direct(&[ItemId(1022)]),
                ItemSet::Direct(&[ItemId(1022)]),
                ItemSet::Direct(&[ItemId(1022)]),
            ],
            product: Product::of(1),
        },
        Craft::Shapeless {
            ingredients: &[ItemSet::Direct(&[ItemId(130)])],
            product: Product::of(9),
        },
    ];
    type Behavior = crate::family::Plain;
}
