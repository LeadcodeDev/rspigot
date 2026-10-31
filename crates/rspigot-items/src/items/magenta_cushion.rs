use rspigot_blocks::BlockId;

use crate::{
    AttributeModifier, Craft, EquipmentSlot, Food, Fuel, Item, ItemId, ItemSet, Product, Rarity,
    Tool,
};

pub struct MagentaCushion;

impl Item for MagentaCushion {
    const NAME: &'static str = "minecraft:magenta_cushion";
    const ID: ItemId = ItemId(1172);
    const CLASS: &'static str = "CushionItem";
    const MAX_STACK_SIZE: u8 = 16;
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
    const COOKING_FUEL: Option<Fuel> = Some(Fuel { burn_time: 50, fast_burn_time: 25 });
    const CRAFTS: &'static [Craft] = &[
        Craft::Shapeless {
            ingredients: &[
                ItemSet::Direct(&[ItemId(1201)]),
                ItemSet::Direct(&[
                    ItemId(1170),
                    ItemId(1171),
                    ItemId(1173),
                    ItemId(1174),
                    ItemId(1175),
                    ItemId(1176),
                    ItemId(1177),
                    ItemId(1178),
                    ItemId(1179),
                    ItemId(1180),
                    ItemId(1181),
                    ItemId(1182),
                    ItemId(1183),
                    ItemId(1184),
                    ItemId(1185),
                ]),
            ],
            product: Product::of(1),
        },
        Craft::Shaped {
            width: 3,
            height: 1,
            symmetrical: true,
            cells: &[
                Some(ItemSet::Direct(&[ItemId(284)])),
                Some(ItemSet::Direct(&[ItemId(284)])),
                Some(ItemSet::Direct(&[ItemId(284)])),
            ],
            product: Product::of(1),
        },
    ];
    type Behavior = crate::family::Plain;
}
