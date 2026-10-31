use rspigot_blocks::BlockId;

use crate::{
    AttributeModifier, Craft, EquipmentSlot, Food, Item, ItemId, ItemSet, Product, Rarity, Tool,
};

pub struct LightGrayCarpet;

impl Item for LightGrayCarpet {
    const NAME: &'static str = "minecraft:light_gray_carpet";
    const ID: ItemId = ItemId(588);
    const CLASS: &'static str = "BlockItem";
    const MAX_STACK_SIZE: u8 = 64;
    const MAX_DAMAGE: Option<u32> = None;
    const WEAPON_DAMAGE: Option<u32> = None;
    const RARITY: Rarity = Rarity::Common;
    const BLOCK: Option<BlockId> = Some(BlockId(600));
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
                ItemSet::Direct(&[ItemId(1207)]),
                ItemSet::Direct(&[
                    ItemId(580),
                    ItemId(581),
                    ItemId(582),
                    ItemId(583),
                    ItemId(584),
                    ItemId(585),
                    ItemId(586),
                    ItemId(587),
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
            cells: &[Some(ItemSet::Direct(&[ItemId(258)])), Some(ItemSet::Direct(&[ItemId(258)]))],
            product: Product::of(3),
        },
    ];
    type Behavior = crate::family::BlockItem;
}
