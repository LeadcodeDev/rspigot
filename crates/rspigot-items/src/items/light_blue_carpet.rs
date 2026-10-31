use rspigot_blocks::BlockId;

use crate::{
    AttributeModifier, Craft, EquipmentSlot, Food, Item, ItemId, ItemSet, Product, Rarity, Tool,
};

pub struct LightBlueCarpet;

impl Item for LightBlueCarpet {
    const NAME: &'static str = "minecraft:light_blue_carpet";
    const ID: ItemId = ItemId(583);
    const CLASS: &'static str = "BlockItem";
    const MAX_STACK_SIZE: u8 = 64;
    const MAX_DAMAGE: Option<u32> = None;
    const WEAPON_DAMAGE: Option<u32> = None;
    const RARITY: Rarity = Rarity::Common;
    const BLOCK: Option<BlockId> = Some(BlockId(595));
    const TOOL: Option<Tool> = None;
    const FOOD: Option<Food> = None;
    const EQUIPPABLE: Option<EquipmentSlot> = Some(EquipmentSlot::Body);
    const ENCHANTABLE: Option<u32> = None;
    const REPAIRABLE: Option<ItemSet> = None;
    const DAMAGE_RESISTANT: Option<&'static str> = None;
    const USE_REMAINDER: Option<ItemId> = None;
    const ATTRIBUTE_MODIFIERS: &'static [AttributeModifier] = &[];
    const CRAFTS: &'static [Craft] = &[
        Craft::Shapeless {
            ingredients: &[
                ItemSet::Direct(&[ItemId(1202)]),
                ItemSet::Direct(&[
                    ItemId(580),
                    ItemId(581),
                    ItemId(582),
                    ItemId(584),
                    ItemId(585),
                    ItemId(586),
                    ItemId(587),
                    ItemId(588),
                    ItemId(589),
                    ItemId(590),
                    ItemId(591),
                    ItemId(592),
                    ItemId(593),
                    ItemId(594),
                    ItemId(595),
                ]),
            ],
            product: Product::of(1),
        },
        Craft::Shaped {
            width: 2,
            height: 1,
            symmetrical: true,
            cells: &[Some(ItemSet::Direct(&[ItemId(253)])), Some(ItemSet::Direct(&[ItemId(253)]))],
            product: Product::of(3),
        },
    ];
    type Behavior = crate::family::BlockItem;
}
