use rspigot_blocks::BlockId;

use crate::{
    AttributeModifier, Craft, EquipmentSlot, Food, Fuel, Item, ItemId, ItemSet, Product, Rarity,
    Tool,
};

pub struct CyanCarpet;

impl Item for CyanCarpet {
    const NAME: &'static str = "minecraft:cyan_carpet";
    const ID: ItemId = ItemId(589);
    const CLASS: &'static str = "BlockItem";
    const MAX_STACK_SIZE: u8 = 64;
    const MAX_DAMAGE: Option<u32> = None;
    const WEAPON_DAMAGE: Option<u32> = None;
    const RARITY: Rarity = Rarity::Common;
    const BLOCK: Option<BlockId> = Some(BlockId(601));
    const TOOL: Option<Tool> = None;
    const FOOD: Option<Food> = None;
    const EQUIPPABLE: Option<EquipmentSlot> = Some(EquipmentSlot::Body);
    const ENCHANTABLE: Option<u32> = None;
    const REPAIRABLE: Option<ItemSet> = None;
    const DAMAGE_RESISTANT: Option<&'static str> = None;
    const USE_REMAINDER: Option<ItemId> = None;
    const ATTRIBUTE_MODIFIERS: &'static [AttributeModifier] = &[];
    const COOKING_FUEL: Option<Fuel> = Some(Fuel { burn_time: 67, fast_burn_time: 33 });
    const CRAFTS: &'static [Craft] = &[
        Craft::Shaped {
            id: "minecraft:cyan_carpet",
            width: 2,
            height: 1,
            symmetrical: true,
            cells: &[Some(ItemSet::Direct(&[ItemId(259)])), Some(ItemSet::Direct(&[ItemId(259)]))],
            product: Product::of(3),
        },
        Craft::Shapeless {
            id: "minecraft:dye_cyan_carpet",
            ingredients: &[
                ItemSet::Direct(&[ItemId(1208)]),
                ItemSet::Direct(&[
                    ItemId(580),
                    ItemId(581),
                    ItemId(582),
                    ItemId(583),
                    ItemId(584),
                    ItemId(585),
                    ItemId(586),
                    ItemId(587),
                    ItemId(588),
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
    ];
    type Behavior = crate::family::BlockItem;
}
