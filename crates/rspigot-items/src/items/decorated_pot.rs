use rspigot_blocks::BlockId;

use crate::{
    AttributeModifier, Craft, EquipmentSlot, Food, Item, ItemId, ItemSet, Product, Rarity, Tool,
    tags,
};

pub struct DecoratedPot;

impl Item for DecoratedPot {
    const NAME: &'static str = "minecraft:decorated_pot";
    const ID: ItemId = ItemId(392);
    const CLASS: &'static str = "BlockItem";
    const MAX_STACK_SIZE: u8 = 64;
    const MAX_DAMAGE: Option<u32> = None;
    const WEAPON_DAMAGE: Option<u32> = None;
    const RARITY: Rarity = Rarity::Common;
    const BLOCK: Option<BlockId> = Some(BlockId(1273));
    const TOOL: Option<Tool> = None;
    const FOOD: Option<Food> = None;
    const EQUIPPABLE: Option<EquipmentSlot> = None;
    const ENCHANTABLE: Option<u32> = None;
    const REPAIRABLE: Option<ItemSet> = None;
    const DAMAGE_RESISTANT: Option<&'static str> = None;
    const USE_REMAINDER: Option<ItemId> = None;
    const ATTRIBUTE_MODIFIERS: &'static [AttributeModifier] = &[];
    const CRAFTS: &'static [Craft] = &[
        Craft::DecoratedPot {
            back: ItemSet::Tag(&tags::ITEM_DECORATED_POT_INGREDIENTS),
            front: ItemSet::Tag(&tags::ITEM_DECORATED_POT_INGREDIENTS),
            left: ItemSet::Tag(&tags::ITEM_DECORATED_POT_INGREDIENTS),
            right: ItemSet::Tag(&tags::ITEM_DECORATED_POT_INGREDIENTS),
        },
        Craft::Shaped {
            width: 3,
            height: 3,
            symmetrical: true,
            cells: &[
                None,
                Some(ItemSet::Direct(&[ItemId(1142)])),
                None,
                Some(ItemSet::Direct(&[ItemId(1142)])),
                None,
                Some(ItemSet::Direct(&[ItemId(1142)])),
                None,
                Some(ItemSet::Direct(&[ItemId(1142)])),
                None,
            ],
            product: Product::of(1),
        },
    ];
    type Behavior = crate::family::BlockItem;
}
