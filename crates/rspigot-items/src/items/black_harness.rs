use rspigot_blocks::BlockId;

use crate::{
    AttributeModifier, Craft, EquipmentSlot, Food, Item, ItemId, ItemSet, Product, Rarity, Tool,
};

pub struct BlackHarness;

impl Item for BlackHarness {
    const NAME: &'static str = "minecraft:black_harness";
    const ID: ItemId = ItemId(965);
    const CLASS: &'static str = "Item";
    const MAX_STACK_SIZE: u8 = 1;
    const MAX_DAMAGE: Option<u32> = None;
    const WEAPON_DAMAGE: Option<u32> = None;
    const RARITY: Rarity = Rarity::Common;
    const BLOCK: Option<BlockId> = None;
    const TOOL: Option<Tool> = None;
    const FOOD: Option<Food> = None;
    const EQUIPPABLE: Option<EquipmentSlot> = Some(EquipmentSlot::Body);
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
                Some(ItemSet::Direct(&[ItemId(1133)])),
                Some(ItemSet::Direct(&[ItemId(1133)])),
                Some(ItemSet::Direct(&[ItemId(1133)])),
                Some(ItemSet::Direct(&[ItemId(231)])),
                Some(ItemSet::Direct(&[ItemId(265)])),
                Some(ItemSet::Direct(&[ItemId(231)])),
            ],
            product: Product::of(1),
        },
        Craft::Shapeless {
            ingredients: &[
                ItemSet::Direct(&[ItemId(1214)]),
                ItemSet::Direct(&[
                    ItemId(950),
                    ItemId(951),
                    ItemId(952),
                    ItemId(953),
                    ItemId(954),
                    ItemId(955),
                    ItemId(956),
                    ItemId(957),
                    ItemId(958),
                    ItemId(959),
                    ItemId(960),
                    ItemId(961),
                    ItemId(962),
                    ItemId(963),
                    ItemId(964),
                ]),
            ],
            product: Product::of(1),
        },
    ];
    type Behavior = crate::family::Plain;
}
