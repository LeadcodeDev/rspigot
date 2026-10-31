use rspigot_blocks::BlockId;

use crate::{
    AttributeModifier, Craft, EquipmentSlot, Food, Fuel, Item, ItemId, ItemSet, Product, Rarity,
    Tool,
};

pub struct GreenWoolStairs;

impl Item for GreenWoolStairs {
    const NAME: &'static str = "minecraft:green_wool_stairs";
    const ID: ItemId = ItemId(279);
    const CLASS: &'static str = "BlockItem";
    const MAX_STACK_SIZE: u8 = 64;
    const MAX_DAMAGE: Option<u32> = None;
    const WEAPON_DAMAGE: Option<u32> = None;
    const RARITY: Rarity = Rarity::Common;
    const BLOCK: Option<BlockId> = Some(BlockId(180));
    const TOOL: Option<Tool> = None;
    const FOOD: Option<Food> = None;
    const EQUIPPABLE: Option<EquipmentSlot> = None;
    const ENCHANTABLE: Option<u32> = None;
    const REPAIRABLE: Option<ItemSet> = None;
    const DAMAGE_RESISTANT: Option<&'static str> = None;
    const USE_REMAINDER: Option<ItemId> = None;
    const ATTRIBUTE_MODIFIERS: &'static [AttributeModifier] = &[];
    const COOKING_FUEL: Option<Fuel> = Some(Fuel { burn_time: 100, fast_burn_time: 50 });
    const CRAFTS: &'static [Craft] = &[
        Craft::Shapeless {
            ingredients: &[
                ItemSet::Direct(&[ItemId(1212)]),
                ItemSet::Direct(&[
                    ItemId(266),
                    ItemId(267),
                    ItemId(268),
                    ItemId(269),
                    ItemId(270),
                    ItemId(271),
                    ItemId(272),
                    ItemId(273),
                    ItemId(274),
                    ItemId(275),
                    ItemId(276),
                    ItemId(277),
                    ItemId(278),
                    ItemId(280),
                    ItemId(281),
                ]),
            ],
            product: Product::of(1),
        },
        Craft::Shaped {
            width: 3,
            height: 3,
            symmetrical: false,
            cells: &[
                Some(ItemSet::Direct(&[ItemId(263)])),
                None,
                None,
                Some(ItemSet::Direct(&[ItemId(263)])),
                Some(ItemSet::Direct(&[ItemId(263)])),
                None,
                Some(ItemSet::Direct(&[ItemId(263)])),
                Some(ItemSet::Direct(&[ItemId(263)])),
                Some(ItemSet::Direct(&[ItemId(263)])),
            ],
            product: Product::of(4),
        },
    ];
    type Behavior = crate::family::BlockItem;
}
