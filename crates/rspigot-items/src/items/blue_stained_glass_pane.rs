use rspigot_blocks::BlockId;

use crate::{
    AttributeModifier, Craft, EquipmentSlot, Food, Item, ItemId, ItemSet, Product, Rarity, Tool,
};

pub struct BlueStainedGlassPane;

impl Item for BlueStainedGlassPane {
    const NAME: &'static str = "minecraft:blue_stained_glass_pane";
    const ID: ItemId = ItemId(632);
    const CLASS: &'static str = "BlockItem";
    const MAX_STACK_SIZE: u8 = 64;
    const MAX_DAMAGE: Option<u32> = None;
    const WEAPON_DAMAGE: Option<u32> = None;
    const RARITY: Rarity = Rarity::Common;
    const BLOCK: Option<BlockId> = Some(BlockId(564));
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
            id: "minecraft:blue_stained_glass_pane",
            width: 3,
            height: 2,
            symmetrical: true,
            cells: &[
                Some(ItemSet::Direct(&[ItemId(616)])),
                Some(ItemSet::Direct(&[ItemId(616)])),
                Some(ItemSet::Direct(&[ItemId(616)])),
                Some(ItemSet::Direct(&[ItemId(616)])),
                Some(ItemSet::Direct(&[ItemId(616)])),
                Some(ItemSet::Direct(&[ItemId(616)])),
            ],
            product: Product::of(16),
        },
        Craft::Shaped {
            id: "minecraft:blue_stained_glass_pane_from_glass_pane",
            width: 3,
            height: 3,
            symmetrical: true,
            cells: &[
                Some(ItemSet::Direct(&[ItemId(482)])),
                Some(ItemSet::Direct(&[ItemId(482)])),
                Some(ItemSet::Direct(&[ItemId(482)])),
                Some(ItemSet::Direct(&[ItemId(482)])),
                Some(ItemSet::Direct(&[ItemId(1210)])),
                Some(ItemSet::Direct(&[ItemId(482)])),
                Some(ItemSet::Direct(&[ItemId(482)])),
                Some(ItemSet::Direct(&[ItemId(482)])),
                Some(ItemSet::Direct(&[ItemId(482)])),
            ],
            product: Product::of(8),
        },
    ];
    type Behavior = crate::family::BlockItem;
}
