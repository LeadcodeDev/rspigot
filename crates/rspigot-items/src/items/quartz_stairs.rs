use rspigot_blocks::BlockId;

use crate::{
    AttributeModifier, Craft, EquipmentSlot, Food, Item, ItemId, ItemSet, Product, Rarity, Tool,
};

pub struct QuartzStairs;

impl Item for QuartzStairs {
    const NAME: &'static str = "minecraft:quartz_stairs";
    const ID: ItemId = ItemId(560);
    const CLASS: &'static str = "BlockItem";
    const MAX_STACK_SIZE: u8 = 64;
    const MAX_DAMAGE: Option<u32> = None;
    const WEAPON_DAMAGE: Option<u32> = None;
    const RARITY: Rarity = Rarity::Common;
    const BLOCK: Option<BlockId> = Some(BlockId(534));
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
            id: "minecraft:quartz_stairs",
            width: 3,
            height: 3,
            symmetrical: false,
            cells: &[
                Some(ItemSet::Direct(&[ItemId(556), ItemId(557), ItemId(559)])),
                None,
                None,
                Some(ItemSet::Direct(&[ItemId(556), ItemId(557), ItemId(559)])),
                Some(ItemSet::Direct(&[ItemId(556), ItemId(557), ItemId(559)])),
                None,
                Some(ItemSet::Direct(&[ItemId(556), ItemId(557), ItemId(559)])),
                Some(ItemSet::Direct(&[ItemId(556), ItemId(557), ItemId(559)])),
                Some(ItemSet::Direct(&[ItemId(556), ItemId(557), ItemId(559)])),
            ],
            product: Product::of(4),
        },
        Craft::Stonecutting {
            id: "minecraft:quartz_stairs_from_quartz_block_stonecutting",
            ingredient: ItemSet::Direct(&[ItemId(557)]),
            count: 1,
        },
    ];
    type Behavior = crate::family::BlockItem;
}
