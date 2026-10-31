use rspigot_blocks::BlockId;

use crate::{
    AttributeModifier, Craft, EquipmentSlot, Food, Item, ItemId, ItemSet, Product, Rarity, Tool,
};

pub struct LimeWoolSlab;

impl Item for LimeWoolSlab {
    const NAME: &'static str = "minecraft:lime_wool_slab";
    const ID: ItemId = ItemId(287);
    const CLASS: &'static str = "BlockItem";
    const MAX_STACK_SIZE: u8 = 64;
    const MAX_DAMAGE: Option<u32> = None;
    const WEAPON_DAMAGE: Option<u32> = None;
    const RARITY: Rarity = Rarity::Common;
    const BLOCK: Option<BlockId> = Some(BlockId(188));
    const TOOL: Option<Tool> = None;
    const FOOD: Option<Food> = None;
    const EQUIPPABLE: Option<EquipmentSlot> = None;
    const ENCHANTABLE: Option<u32> = None;
    const REPAIRABLE: Option<ItemSet> = None;
    const DAMAGE_RESISTANT: Option<&'static str> = None;
    const USE_REMAINDER: Option<ItemId> = None;
    const ATTRIBUTE_MODIFIERS: &'static [AttributeModifier] = &[];
    const CRAFTS: &'static [Craft] = &[
        Craft::Shapeless {
            ingredients: &[
                ItemSet::Direct(&[ItemId(1204)]),
                ItemSet::Direct(&[
                    ItemId(282),
                    ItemId(283),
                    ItemId(284),
                    ItemId(285),
                    ItemId(286),
                    ItemId(288),
                    ItemId(289),
                    ItemId(290),
                    ItemId(291),
                    ItemId(292),
                    ItemId(293),
                    ItemId(294),
                    ItemId(295),
                    ItemId(296),
                    ItemId(297),
                ]),
            ],
            product: Product::of(1),
        },
        Craft::Shaped {
            width: 3,
            height: 1,
            symmetrical: true,
            cells: &[
                Some(ItemSet::Direct(&[ItemId(255)])),
                Some(ItemSet::Direct(&[ItemId(255)])),
                Some(ItemSet::Direct(&[ItemId(255)])),
            ],
            product: Product::of(6),
        },
    ];
    type Behavior = crate::family::BlockItem;
}
