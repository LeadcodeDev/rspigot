use rspigot_blocks::BlockId;

use crate::{EquipmentSlot, Food, Item, ItemId, ItemSet, Rarity, Tool};

pub struct ZombieHorseSpawnEgg;

impl Item for ZombieHorseSpawnEgg {
    const NAME: &'static str = "minecraft:zombie_horse_spawn_egg";
    const ID: ItemId = ItemId(1334);
    const CLASS: &'static str = "SpawnEggItem";
    const MAX_STACK_SIZE: u8 = 64;
    const MAX_DAMAGE: Option<u32> = None;
    const RARITY: Rarity = Rarity::Common;
    const BLOCK: Option<BlockId> = None;
    const TOOL: Option<Tool> = None;
    const FOOD: Option<Food> = None;
    const EQUIPPABLE: Option<EquipmentSlot> = None;
    const ENCHANTABLE: Option<u32> = None;
    const REPAIRABLE: Option<ItemSet> = None;
    const DAMAGE_RESISTANT: Option<&'static str> = None;
    const USE_REMAINDER: Option<ItemId> = None;
    type Behavior = crate::family::Plain;
}
