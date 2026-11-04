use rspigot_blocks::BlockId;

use crate::{
    AttributeModifier, Craft, EquipmentSlot, Food, Fuel, Item, ItemId, ItemSet, Product, Rarity,
    Tool,
};

pub struct Bow;

impl Item for Bow {
    const NAME: &'static str = "minecraft:bow";
    const ID: ItemId = ItemId(1008);
    const CLASS: &'static str = "BowItem";
    const MAX_STACK_SIZE: u8 = 1;
    const MAX_DAMAGE: Option<u32> = Some(384);
    const WEAPON_DAMAGE: Option<u32> = None;
    const RARITY: Rarity = Rarity::Common;
    const BLOCK: Option<BlockId> = None;
    const TOOL: Option<Tool> = None;
    const FOOD: Option<Food> = None;
    const EQUIPPABLE: Option<EquipmentSlot> = None;
    const ENCHANTABLE: Option<u32> = Some(1);
    const REPAIRABLE: Option<ItemSet> = None;
    const DAMAGE_RESISTANT: Option<&'static str> = None;
    const USE_REMAINDER: Option<ItemId> = None;
    const ATTRIBUTE_MODIFIERS: &'static [AttributeModifier] = &[];
    const COOKING_FUEL: Option<Fuel> = Some(Fuel { burn_time: 300, fast_burn_time: 150 });
    const CRAFTS: &'static [Craft] = &[
        Craft::Shaped {
            id: "minecraft:bow",
            width: 3,
            height: 3,
            symmetrical: false,
            cells: &[
                None,
                Some(ItemSet::Direct(&[ItemId(1060)])),
                Some(ItemSet::Direct(&[ItemId(1062)])),
                Some(ItemSet::Direct(&[ItemId(1060)])),
                None,
                Some(ItemSet::Direct(&[ItemId(1062)])),
                None,
                Some(ItemSet::Direct(&[ItemId(1060)])),
                Some(ItemSet::Direct(&[ItemId(1062)])),
            ],
            product: Product::of(1),
        },
        Craft::RepairItem { id: "minecraft:repair_item" },
    ];
    type Behavior = crate::family::Plain;
}
