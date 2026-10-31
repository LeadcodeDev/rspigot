use rspigot_blocks::BlockId;

use crate::{
    AttributeModifier, Craft, EquipmentSlot, FireworkShapes, Food, Item, ItemId, ItemSet, Rarity,
    Tool, tags,
};

pub struct FireworkStar;

impl Item for FireworkStar {
    const NAME: &'static str = "minecraft:firework_star";
    const ID: ItemId = ItemId(1394);
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
    const DAMAGE_RESISTANT: Option<&'static str> = None;
    const USE_REMAINDER: Option<ItemId> = None;
    const ATTRIBUTE_MODIFIERS: &'static [AttributeModifier] = &[];
    const CRAFTS: &'static [Craft] = &[
        Craft::FireworkStar {
            dye: ItemSet::Tag(&tags::ITEM_DYES),
            fuel: ItemSet::Direct(&[ItemId(1064)]),
            trail: ItemSet::Direct(&[ItemId(1012)]),
            twinkle: ItemSet::Direct(&[ItemId(1189)]),
            shapes: FireworkShapes {
                burst: ItemSet::Direct(&[ItemId(1063)]),
                creeper: ItemSet::Tag(&tags::ITEM_SKULLS),
                large_ball: ItemSet::Direct(&[ItemId(1369)]),
                star: ItemSet::Direct(&[ItemId(1268)]),
            },
        },
        Craft::FireworkStarFade { dye: ItemSet::Tag(&tags::ITEM_DYES) },
    ];
    type Behavior = crate::family::Plain;
}
