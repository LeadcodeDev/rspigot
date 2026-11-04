use rspigot_blocks::BlockId;

use crate::{
    AttributeModifier, Craft, EquipmentSlot, Food, Fuel, Item, ItemId, ItemSet, Product, Rarity,
    Tool,
};

pub struct OrangeWool;

impl Item for OrangeWool {
    const NAME: &'static str = "minecraft:orange_wool";
    const ID: ItemId = ItemId(251);
    const CLASS: &'static str = "BlockItem";
    const MAX_STACK_SIZE: u8 = 64;
    const MAX_DAMAGE: Option<u32> = None;
    const WEAPON_DAMAGE: Option<u32> = None;
    const RARITY: Rarity = Rarity::Common;
    const BLOCK: Option<BlockId> = Some(BlockId(152));
    const TOOL: Option<Tool> = None;
    const FOOD: Option<Food> = None;
    const EQUIPPABLE: Option<EquipmentSlot> = None;
    const ENCHANTABLE: Option<u32> = None;
    const REPAIRABLE: Option<ItemSet> = None;
    const DAMAGE_RESISTANT: Option<&'static str> = None;
    const USE_REMAINDER: Option<ItemId> = None;
    const ATTRIBUTE_MODIFIERS: &'static [AttributeModifier] = &[];
    const COOKING_FUEL: Option<Fuel> = Some(Fuel { burn_time: 100, fast_burn_time: 50 });
    const CRAFTS: &'static [Craft] = &[Craft::Shapeless {
        id: "minecraft:dye_orange_wool",
        ingredients: &[
            ItemSet::Direct(&[ItemId(1200)]),
            ItemSet::Direct(&[
                ItemId(250),
                ItemId(252),
                ItemId(253),
                ItemId(254),
                ItemId(255),
                ItemId(256),
                ItemId(257),
                ItemId(258),
                ItemId(259),
                ItemId(260),
                ItemId(261),
                ItemId(262),
                ItemId(263),
                ItemId(264),
                ItemId(265),
            ]),
        ],
        product: Product::of(1),
    }];
    type Behavior = crate::family::BlockItem;
}
