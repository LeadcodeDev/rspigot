use rspigot_blocks::BlockId;

use crate::{
    AttributeModifier, Craft, EquipmentSlot, Food, Fuel, Item, ItemId, ItemSet, Product, Rarity,
    Tool,
};

pub struct OrangeBanner;

impl Item for OrangeBanner {
    const NAME: &'static str = "minecraft:orange_banner";
    const ID: ItemId = ItemId(1418);
    const CLASS: &'static str = "BannerItem";
    const MAX_STACK_SIZE: u8 = 16;
    const MAX_DAMAGE: Option<u32> = None;
    const WEAPON_DAMAGE: Option<u32> = None;
    const RARITY: Rarity = Rarity::Common;
    const BLOCK: Option<BlockId> = Some(BlockId(618));
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
            id: "minecraft:orange_banner",
            width: 3,
            height: 3,
            symmetrical: true,
            cells: &[
                Some(ItemSet::Direct(&[ItemId(251)])),
                Some(ItemSet::Direct(&[ItemId(251)])),
                Some(ItemSet::Direct(&[ItemId(251)])),
                Some(ItemSet::Direct(&[ItemId(251)])),
                Some(ItemSet::Direct(&[ItemId(251)])),
                Some(ItemSet::Direct(&[ItemId(251)])),
                None,
                Some(ItemSet::Direct(&[ItemId(1060)])),
                None,
            ],
            product: Product::of(1),
        },
        Craft::BannerDuplicate { id: "minecraft:orange_banner_duplicate" },
    ];
    type Behavior = crate::family::BlockItem;
}
