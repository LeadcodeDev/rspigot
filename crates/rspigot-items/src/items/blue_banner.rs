use rspigot_blocks::BlockId;

use crate::{
    AttributeModifier, Craft, EquipmentSlot, Food, Item, ItemId, ItemSet, Product, Rarity, Tool,
};

pub struct BlueBanner;

impl Item for BlueBanner {
    const NAME: &'static str = "minecraft:blue_banner";
    const ID: ItemId = ItemId(1428);
    const CLASS: &'static str = "BannerItem";
    const MAX_STACK_SIZE: u8 = 16;
    const MAX_DAMAGE: Option<u32> = None;
    const WEAPON_DAMAGE: Option<u32> = None;
    const RARITY: Rarity = Rarity::Common;
    const BLOCK: Option<BlockId> = Some(BlockId(628));
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
            width: 3,
            height: 3,
            symmetrical: true,
            cells: &[
                Some(ItemSet::Direct(&[ItemId(261)])),
                Some(ItemSet::Direct(&[ItemId(261)])),
                Some(ItemSet::Direct(&[ItemId(261)])),
                Some(ItemSet::Direct(&[ItemId(261)])),
                Some(ItemSet::Direct(&[ItemId(261)])),
                Some(ItemSet::Direct(&[ItemId(261)])),
                None,
                Some(ItemSet::Direct(&[ItemId(1060)])),
                None,
            ],
            product: Product::of(1),
        },
        Craft::BannerDuplicate,
    ];
    type Behavior = crate::family::BlockItem;
}
