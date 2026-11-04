use rspigot_blocks::BlockId;

use crate::{
    AttributeModifier, Craft, EquipmentSlot, Food, Fuel, Item, ItemId, ItemSet, Product, Rarity,
    Tool,
};

pub struct LightGrayBanner;

impl Item for LightGrayBanner {
    const NAME: &'static str = "minecraft:light_gray_banner";
    const ID: ItemId = ItemId(1425);
    const CLASS: &'static str = "BannerItem";
    const MAX_STACK_SIZE: u8 = 16;
    const MAX_DAMAGE: Option<u32> = None;
    const WEAPON_DAMAGE: Option<u32> = None;
    const RARITY: Rarity = Rarity::Common;
    const BLOCK: Option<BlockId> = Some(BlockId(625));
    const TOOL: Option<Tool> = None;
    const FOOD: Option<Food> = None;
    const EQUIPPABLE: Option<EquipmentSlot> = None;
    const ENCHANTABLE: Option<u32> = None;
    const REPAIRABLE: Option<ItemSet> = None;
    const DAMAGE_RESISTANT: Option<&'static str> = None;
    const USE_REMAINDER: Option<ItemId> = None;
    const ATTRIBUTE_MODIFIERS: &'static [AttributeModifier] = &[];
    const COOKING_FUEL: Option<Fuel> = Some(Fuel { burn_time: 300, fast_burn_time: 150 });
    const CRAFTS: &'static [Craft] = &[
        Craft::Shaped {
            id: "minecraft:light_gray_banner",
            width: 3,
            height: 3,
            symmetrical: true,
            cells: &[
                Some(ItemSet::Direct(&[ItemId(258)])),
                Some(ItemSet::Direct(&[ItemId(258)])),
                Some(ItemSet::Direct(&[ItemId(258)])),
                Some(ItemSet::Direct(&[ItemId(258)])),
                Some(ItemSet::Direct(&[ItemId(258)])),
                Some(ItemSet::Direct(&[ItemId(258)])),
                None,
                Some(ItemSet::Direct(&[ItemId(1060)])),
                None,
            ],
            product: Product::of(1),
        },
        Craft::BannerDuplicate { id: "minecraft:light_gray_banner_duplicate" },
    ];
    type Behavior = crate::family::BlockItem;
}
