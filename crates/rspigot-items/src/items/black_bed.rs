use rspigot_blocks::BlockId;

use crate::{
    AttributeModifier, Craft, EquipmentSlot, Food, Item, ItemId, ItemSet, Product, Rarity, Tool,
    tags,
};

pub struct BlackBed;

impl Item for BlackBed {
    const NAME: &'static str = "minecraft:black_bed";
    const ID: ItemId = ItemId(1234);
    const CLASS: &'static str = "BlockItem";
    const MAX_STACK_SIZE: u8 = 1;
    const MAX_DAMAGE: Option<u32> = None;
    const WEAPON_DAMAGE: Option<u32> = None;
    const RARITY: Rarity = Rarity::Common;
    const BLOCK: Option<BlockId> = Some(BlockId(134));
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
            height: 2,
            symmetrical: true,
            cells: &[
                Some(ItemSet::Direct(&[ItemId(265)])),
                Some(ItemSet::Direct(&[ItemId(265)])),
                Some(ItemSet::Direct(&[ItemId(265)])),
                Some(ItemSet::Tag(&tags::ITEM_PLANKS)),
                Some(ItemSet::Tag(&tags::ITEM_PLANKS)),
                Some(ItemSet::Tag(&tags::ITEM_PLANKS)),
            ],
            product: Product::of(1),
        },
        Craft::Shapeless {
            ingredients: &[
                ItemSet::Direct(&[ItemId(1214)]),
                ItemSet::Direct(&[
                    ItemId(1219),
                    ItemId(1220),
                    ItemId(1221),
                    ItemId(1222),
                    ItemId(1223),
                    ItemId(1224),
                    ItemId(1225),
                    ItemId(1226),
                    ItemId(1227),
                    ItemId(1228),
                    ItemId(1229),
                    ItemId(1230),
                    ItemId(1231),
                    ItemId(1232),
                    ItemId(1233),
                ]),
            ],
            product: Product::of(1),
        },
    ];
    type Behavior = crate::family::BlockItem;
}
