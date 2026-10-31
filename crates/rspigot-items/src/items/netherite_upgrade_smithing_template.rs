use rspigot_blocks::BlockId;

use crate::{
    AttributeModifier, Craft, EquipmentSlot, Food, Item, ItemId, ItemSet, Product, Rarity, Tool,
};

pub struct NetheriteUpgradeSmithingTemplate;

impl Item for NetheriteUpgradeSmithingTemplate {
    const NAME: &'static str = "minecraft:netherite_upgrade_smithing_template";
    const ID: ItemId = ItemId(1579);
    const CLASS: &'static str = "SmithingTemplateItem";
    const MAX_STACK_SIZE: u8 = 64;
    const MAX_DAMAGE: Option<u32> = None;
    const WEAPON_DAMAGE: Option<u32> = None;
    const RARITY: Rarity = Rarity::Uncommon;
    const BLOCK: Option<BlockId> = None;
    const TOOL: Option<Tool> = None;
    const FOOD: Option<Food> = None;
    const EQUIPPABLE: Option<EquipmentSlot> = None;
    const ENCHANTABLE: Option<u32> = None;
    const REPAIRABLE: Option<ItemSet> = None;
    const DAMAGE_RESISTANT: Option<&'static str> = None;
    const USE_REMAINDER: Option<ItemId> = None;
    const ATTRIBUTE_MODIFIERS: &'static [AttributeModifier] = &[];
    const CRAFTS: &'static [Craft] = &[Craft::Shaped {
        width: 3,
        height: 3,
        symmetrical: true,
        cells: &[
            Some(ItemSet::Direct(&[ItemId(1012)])),
            Some(ItemSet::Direct(&[ItemId(1579)])),
            Some(ItemSet::Direct(&[ItemId(1012)])),
            Some(ItemSet::Direct(&[ItemId(1012)])),
            Some(ItemSet::Direct(&[ItemId(433)])),
            Some(ItemSet::Direct(&[ItemId(1012)])),
            Some(ItemSet::Direct(&[ItemId(1012)])),
            Some(ItemSet::Direct(&[ItemId(1012)])),
            Some(ItemSet::Direct(&[ItemId(1012)])),
        ],
        product: Product::of(2),
    }];
    type Behavior = crate::family::Plain;
}
