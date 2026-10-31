use rspigot_blocks::BlockId;

use crate::{
    AttributeModifier, Craft, EquipmentSlot, Food, Item, ItemId, ItemSet, Product, Rarity, Tool,
};

pub struct FireworkRocket;

impl Item for FireworkRocket {
    const NAME: &'static str = "minecraft:firework_rocket";
    const ID: ItemId = ItemId(1393);
    const CLASS: &'static str = "FireworkRocketItem";
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
        Craft::FireworkRocket {
            shell: ItemSet::Direct(&[ItemId(1145)]),
            star: ItemSet::Direct(&[ItemId(1394)]),
            fuel: ItemSet::Direct(&[ItemId(1064)]),
            count: 3,
        },
        Craft::Shapeless {
            ingredients: &[ItemSet::Direct(&[ItemId(1064)]), ItemSet::Direct(&[ItemId(1145)])],
            product: Product::of(3),
        },
    ];
    type Behavior = crate::family::Plain;
}
