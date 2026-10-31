use rspigot_blocks::BlockId;

use crate::{
    AttributeModifier, Craft, EquipmentSlot, Food, Item, ItemId, ItemSet, Product, Rarity, Tool,
};

pub struct PolishedBlackstoneBricks;

impl Item for PolishedBlackstoneBricks {
    const NAME: &'static str = "minecraft:polished_blackstone_bricks";
    const ID: ItemId = ItemId(1545);
    const CLASS: &'static str = "BlockItem";
    const MAX_STACK_SIZE: u8 = 64;
    const MAX_DAMAGE: Option<u32> = None;
    const WEAPON_DAMAGE: Option<u32> = None;
    const RARITY: Rarity = Rarity::Common;
    const BLOCK: Option<BlockId> = Some(BlockId(1019));
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
            width: 2,
            height: 2,
            symmetrical: true,
            cells: &[
                Some(ItemSet::Direct(&[ItemId(1541)])),
                Some(ItemSet::Direct(&[ItemId(1541)])),
                Some(ItemSet::Direct(&[ItemId(1541)])),
                Some(ItemSet::Direct(&[ItemId(1541)])),
            ],
            product: Product::of(4),
        },
        Craft::Stonecutting {
            id: "minecraft:polished_blackstone_bricks_from_blackstone_stonecutting",
            ingredient: ItemSet::Direct(&[ItemId(1537)]),
            count: 1,
        },
        Craft::Stonecutting {
            id: "minecraft:polished_blackstone_bricks_from_polished_blackstone_stonecutting",
            ingredient: ItemSet::Direct(&[ItemId(1541)]),
            count: 1,
        },
    ];
    type Behavior = crate::family::BlockItem;
}
