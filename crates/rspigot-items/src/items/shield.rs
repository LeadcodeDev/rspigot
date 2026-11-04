use rspigot_blocks::BlockId;

use crate::{
    AttributeModifier, Craft, EquipmentSlot, Food, Item, ItemId, ItemSet, Product, Rarity, Tool,
    tags,
};

pub struct Shield;

impl Item for Shield {
    const NAME: &'static str = "minecraft:shield";
    const ID: ItemId = ItemId(1446);
    const CLASS: &'static str = "ShieldItem";
    const MAX_STACK_SIZE: u8 = 1;
    const MAX_DAMAGE: Option<u32> = Some(336);
    const WEAPON_DAMAGE: Option<u32> = None;
    const RARITY: Rarity = Rarity::Common;
    const BLOCK: Option<BlockId> = None;
    const TOOL: Option<Tool> = None;
    const FOOD: Option<Food> = None;
    const EQUIPPABLE: Option<EquipmentSlot> = Some(EquipmentSlot::Offhand);
    const ENCHANTABLE: Option<u32> = None;
    const REPAIRABLE: Option<ItemSet> = Some(ItemSet::Tag(&tags::ITEM_WOODEN_TOOL_MATERIALS));
    const DAMAGE_RESISTANT: Option<&'static str> = None;
    const USE_REMAINDER: Option<ItemId> = None;
    const ATTRIBUTE_MODIFIERS: &'static [AttributeModifier] = &[];
    const CRAFTS: &'static [Craft] = &[
        Craft::RepairItem { id: "minecraft:repair_item" },
        Craft::Shaped {
            id: "minecraft:shield",
            width: 3,
            height: 3,
            symmetrical: true,
            cells: &[
                Some(ItemSet::Tag(&tags::ITEM_WOODEN_TOOL_MATERIALS)),
                Some(ItemSet::Direct(&[ItemId(1018)])),
                Some(ItemSet::Tag(&tags::ITEM_WOODEN_TOOL_MATERIALS)),
                Some(ItemSet::Tag(&tags::ITEM_WOODEN_TOOL_MATERIALS)),
                Some(ItemSet::Tag(&tags::ITEM_WOODEN_TOOL_MATERIALS)),
                Some(ItemSet::Tag(&tags::ITEM_WOODEN_TOOL_MATERIALS)),
                None,
                Some(ItemSet::Tag(&tags::ITEM_WOODEN_TOOL_MATERIALS)),
                None,
            ],
            product: Product::of(1),
        },
        Craft::ShieldDecoration {
            id: "minecraft:shield_decoration",
            banner: ItemSet::Tag(&tags::ITEM_BANNERS),
        },
    ];
    type Behavior = crate::family::Plain;
}
