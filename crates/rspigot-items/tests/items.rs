use rspigot_blocks::{Block, BlockId, BlockKind};
use rspigot_data::{registries, tags};
use rspigot_items::{
    Apple, BlockSet, DEFAULT_MAX_STACK_SIZE, DiamondPickaxe, Elytra, EquipmentSlot, Food,
    IronHelmet, IronSword, Item, ItemId, ItemKind, ItemSet, MushroomStew, NetheriteIngot, OakDoor,
    Rarity, Redstone, Stone, Torch, WoodenPickaxe,
};

fn block(name: &str) -> BlockId {
    BlockKind::from_name(name).unwrap().id()
}

fn registry_item(name: &str) -> ItemId {
    let id = registries::static_index("minecraft:item", name).unwrap();
    ItemId(u16::try_from(id).unwrap())
}

#[test]
fn every_vanilla_item_is_generated_in_registry_order() {
    assert_eq!(ItemKind::ALL.len(), 1658);
    for (index, kind) in ItemKind::ALL.into_iter().enumerate() {
        assert_eq!(usize::from(kind.id().0), index);
        assert_eq!(ItemKind::from_id(kind.id()), Some(kind));
        assert_eq!(kind.id().kind(), Some(kind));
        assert_eq!(ItemKind::from_name(kind.name()), Some(kind));
    }
    assert_eq!(ItemKind::from_id(ItemId(1658)), None);
    assert_eq!(ItemId(u16::MAX).kind(), None);
}

#[test]
fn ids_match_the_registry_report() {
    let entries = registries::static_entries("minecraft:item").unwrap();
    assert_eq!(entries.len(), ItemKind::ALL.len());
    for kind in ItemKind::ALL {
        assert_eq!(entries[usize::from(kind.id().0)], kind.name());
        assert_eq!(registry_item(kind.name()), kind.id());
    }
    assert_eq!(Stone::ID, registry_item("minecraft:stone"));
    assert_eq!(Stone::ID, ItemId(1));
    assert_eq!(ItemId::AIR, ItemKind::Air.id());
    assert_eq!(ItemKind::Air.name(), "minecraft:air");
}

#[test]
fn lookups_by_name() {
    assert_eq!(ItemKind::from_name("stone"), Some(ItemKind::Stone));
    assert_eq!(
        ItemKind::from_name("minecraft:stone"),
        Some(ItemKind::Stone)
    );
    assert_eq!(ItemKind::from_name("other:stone"), None);
    assert_eq!(ItemKind::from_name("nope"), None);
    assert_eq!(ItemKind::from_name(""), None);
}

#[test]
fn stack_sizes_fall_back_to_the_vanilla_default() {
    assert_eq!(ItemKind::Stone.id().max_stack_size(), 64);
    assert_eq!(ItemKind::DiamondSword.id().max_stack_size(), 1);
    assert_eq!(ItemKind::EnderPearl.id().max_stack_size(), 16);
    assert_eq!(ItemKind::Snowball.max_stack_size(), 16);
    assert_eq!(DEFAULT_MAX_STACK_SIZE, 64);
    assert_eq!(ItemId(u16::MAX).max_stack_size(), DEFAULT_MAX_STACK_SIZE);
}

#[test]
fn diamond_pickaxe_matches_vanilla() {
    assert_eq!(DiamondPickaxe::NAME, "minecraft:diamond_pickaxe");
    assert_eq!(DiamondPickaxe::ID, ItemId(1052));
    assert_eq!(DiamondPickaxe::CLASS, "Item");
    assert_eq!(DiamondPickaxe::MAX_DAMAGE, Some(1561));
    assert_eq!(DiamondPickaxe::MAX_STACK_SIZE, 1);
    assert_eq!(DiamondPickaxe::RARITY, Rarity::Common);
    assert_eq!(DiamondPickaxe::ENCHANTABLE, Some(10));
    assert_eq!(DiamondPickaxe::BLOCK, None);
    let tool = DiamondPickaxe::TOOL.unwrap();
    assert!(tool.is_correct_for_drops(block("obsidian")));
    assert!(tool.is_correct_for_drops(block("diamond_ore")));
    assert!(tool.is_correct_for_drops(block("stone")));
    assert!(!tool.is_correct_for_drops(block("dirt")));
    assert_eq!(tool.mining_speed(block("stone")), 8.0);
    assert_eq!(tool.mining_speed(block("dirt")), 1.0);
    assert_eq!(tool.default_mining_speed, 1.0);
    assert_eq!(tool.damage_per_block, 1);
    assert!(tool.can_destroy_blocks_in_creative);
    assert_eq!(tool.rules.len(), 2);
    assert_eq!(
        tool.rules[0].blocks.tag(),
        Some("minecraft:incorrect_for_diamond_tool")
    );
    assert_eq!(tool.rules[0].correct_for_drops, Some(false));
    assert_eq!(tool.rules[0].speed, None);
    assert_eq!(
        tool.rules[1].blocks.tag(),
        Some("minecraft:mineable/pickaxe")
    );
    assert_eq!(ItemKind::DiamondPickaxe.tool(), DiamondPickaxe::TOOL);
    assert_eq!(ItemKind::DiamondPickaxe.max_damage(), Some(1561));
    assert_eq!(ItemKind::DiamondPickaxe.max_stack_size(), 1);
    match DiamondPickaxe::REPAIRABLE {
        Some(ItemSet::Tag(tag)) => {
            assert_eq!(tag.name, "minecraft:diamond_tool_materials");
            assert!(ItemSet::Tag(tag).contains(ItemKind::Diamond.id()));
        }
        other => panic!("{other:?}"),
    }
}

#[test]
fn wooden_pickaxe_cannot_harvest_iron_ore() {
    let tool = WoodenPickaxe::TOOL.unwrap();
    assert!(!tool.is_correct_for_drops(block("iron_ore")));
    assert!(tool.is_correct_for_drops(block("stone")));
    assert_eq!(tool.mining_speed(block("iron_ore")), 2.0);
    assert_eq!(WoodenPickaxe::MAX_DAMAGE, Some(59));
    assert_eq!(WoodenPickaxe::ENCHANTABLE, Some(15));
    let stone = ItemKind::StonePickaxe.tool().unwrap();
    assert!(stone.is_correct_for_drops(block("iron_ore")));
    assert!(!stone.is_correct_for_drops(block("diamond_ore")));
}

#[test]
fn swords_shears_and_mace_rules() {
    let sword = IronSword::TOOL.unwrap();
    assert!(sword.is_correct_for_drops(block("cobweb")));
    assert_eq!(sword.mining_speed(block("cobweb")), 15.0);
    assert_eq!(sword.mining_speed(block("oak_leaves")), 1.5);
    assert_eq!(sword.damage_per_block, 2);
    assert!(!sword.can_destroy_blocks_in_creative);
    assert_eq!(sword.rules[0].blocks, BlockSet::Direct(&[BlockId(139)]));
    assert_eq!(sword.rules[0].blocks.tag(), None);
    assert_eq!(sword.rules[1].speed, Some(f32::MAX));
    let shears = ItemKind::Shears.tool().unwrap();
    assert!(shears.is_correct_for_drops(block("cobweb")));
    assert!(!shears.is_correct_for_drops(block("oak_leaves")));
    assert_eq!(shears.mining_speed(block("oak_leaves")), 15.0);
    let mace = ItemKind::Mace.tool().unwrap();
    assert!(mace.rules.is_empty());
    assert!(!mace.is_correct_for_drops(block("stone")));
    assert_eq!(mace.mining_speed(block("stone")), 1.0);
    assert!(ItemKind::Stick.tool().is_none());
    assert_eq!(
        ItemKind::ALL
            .iter()
            .filter(|kind| kind.tool().is_some())
            .count(),
        38
    );
}

#[test]
fn foods_match_vanilla() {
    assert_eq!(
        Apple::FOOD,
        Some(Food {
            nutrition: 4,
            saturation: 2.4,
            can_always_eat: false
        })
    );
    assert_eq!(ItemKind::Apple.food(), Apple::FOOD);
    assert_eq!(
        ItemKind::GoldenApple.food().map(|food| food.can_always_eat),
        Some(true)
    );
    assert_eq!(MushroomStew::USE_REMAINDER, Some(ItemKind::Bowl.id()));
    assert_eq!(MushroomStew::MAX_STACK_SIZE, 1);
    assert_eq!(ItemKind::Stone.food(), None);
    let foods = ItemKind::ALL
        .iter()
        .filter(|kind| kind.food().is_some())
        .count();
    assert_eq!(foods, 44);
}

#[test]
fn equipment_slots_match_vanilla() {
    assert_eq!(IronHelmet::EQUIPPABLE, Some(EquipmentSlot::Head));
    assert_eq!(IronHelmet::MAX_DAMAGE, Some(165));
    assert_eq!(ItemKind::IronHelmet.equippable(), Some(EquipmentSlot::Head));
    assert_eq!(Elytra::EQUIPPABLE, Some(EquipmentSlot::Chest));
    assert_eq!(Elytra::RARITY, Rarity::Epic);
    assert_eq!(Elytra::REPAIRABLE, Some(ItemSet::Direct(&[ItemId(973)])));
    assert_eq!(ItemKind::PhantomMembrane.id(), ItemId(973));
    assert_eq!(EquipmentSlot::Head.name(), "head");
    assert_eq!(Rarity::Epic.name(), "epic");
    let equippable = ItemKind::ALL
        .iter()
        .filter(|kind| kind.equippable().is_some())
        .count();
    assert_eq!(equippable, 84);
}

#[test]
fn other_components() {
    assert_eq!(NetheriteIngot::DAMAGE_RESISTANT, Some("minecraft:is_fire"));
    assert_eq!(ItemKind::Stone.damage_resistant(), None);
    assert_eq!(ItemKind::EnchantedGoldenApple.rarity(), Rarity::Rare);
    let count =
        |test: fn(&ItemKind) -> bool| ItemKind::ALL.iter().filter(|kind| test(kind)).count();
    assert_eq!(count(|kind| kind.max_damage().is_some()), 84);
    assert_eq!(count(|kind| kind.enchantable().is_some()), 77);
    assert_eq!(count(|kind| kind.repairable().is_some()), 75);
    assert_eq!(count(|kind| kind.damage_resistant().is_some()), 17);
    assert_eq!(count(|kind| kind.use_remainder().is_some()), 7);
    assert_eq!(count(|kind| kind.max_stack_size() == 64), 1342);
    assert_eq!(count(|kind| kind.max_stack_size() == 16), 68);
    assert_eq!(count(|kind| kind.max_stack_size() == 1), 248);
    assert_eq!(count(|kind| kind.rarity() == Rarity::Uncommon), 78);
}

#[test]
fn block_items_place_their_block() {
    assert_eq!(OakDoor::BLOCK, Some(rspigot_blocks::OakDoor::ID));
    assert_eq!(OakDoor::CLASS, "DoubleHighBlockItem");
    assert_eq!(ItemKind::OakDoor.placed_block(), Some(BlockKind::OakDoor));
    assert_eq!(Torch::BLOCK, Some(rspigot_blocks::Torch::ID));
    assert_eq!(Torch::CLASS, "StandingAndWallBlockItem");
    assert_eq!(ItemKind::Torch.placed_block(), Some(BlockKind::Torch));
    assert_eq!(Redstone::BLOCK, Some(rspigot_blocks::RedstoneWire::ID));
    assert_eq!(
        ItemKind::Redstone.placed_block(),
        Some(BlockKind::RedstoneWire)
    );
    assert_eq!(ItemKind::Stone.block(), Some(BlockKind::Stone.id()));
    assert_eq!(ItemKind::DiamondPickaxe.placed_block(), None);
    assert_eq!(ItemKind::Air.placed_block(), None);
}

#[test]
fn from_block_finds_the_placing_item() {
    assert_eq!(ItemKind::from_block(block("stone")), Some(ItemKind::Stone));
    assert_eq!(
        ItemKind::from_block(block("redstone_wire")),
        Some(ItemKind::Redstone)
    );
    assert_eq!(ItemKind::from_block(block("torch")), Some(ItemKind::Torch));
    assert_eq!(ItemKind::from_block(block("wall_torch")), None);
    assert_eq!(ItemKind::from_block(block("water")), None);
    assert_eq!(ItemKind::from_block(BlockId(u16::MAX)), None);
    let mut placing = 0;
    for kind in ItemKind::ALL {
        if let Some(block) = kind.block() {
            placing += 1;
            assert_eq!(ItemKind::from_block(block), Some(kind));
        }
    }
    assert_eq!(placing, 1141);
}

#[test]
fn families_agree_with_block_data() {
    for kind in ItemKind::ALL {
        assert_eq!(kind.placed_block().map(BlockKind::id), kind.block());
        if let Some(block) = kind.placed_block() {
            assert_eq!(BlockKind::from_item(kind.id().0), Some(block));
            assert_eq!(block.item(), Some(kind.id().0));
        }
    }
    for block in BlockKind::ALL {
        if let Some(item) = block.item() {
            assert!(ItemKind::from_id(ItemId(item)).is_some_and(|kind| kind.block().is_some()));
        }
    }
}

#[test]
fn tags_match_the_datapack() {
    let mut seen = 0;
    for kind in ItemKind::ALL {
        for rule in kind.tool().map_or(&[][..], |tool| tool.rules) {
            if let BlockSet::Tag(tag) = rule.blocks {
                seen += 1;
                let expected = tags::tag("minecraft:block", tag.name).unwrap();
                let actual: Vec<u32> = tag.blocks.iter().map(|block| u32::from(block.0)).collect();
                assert_eq!(actual, expected);
            }
        }
        if let Some(ItemSet::Tag(tag)) = kind.repairable() {
            seen += 1;
            let expected = tags::tag("minecraft:item", tag.name).unwrap();
            let actual: Vec<u32> = tag.items.iter().map(|item| u32::from(item.0)).collect();
            assert_eq!(actual, expected);
        }
    }
    assert!(seen > 0);
}

#[test]
fn classes_are_vanilla_simple_names() {
    assert_eq!(ItemKind::Air.class(), "AirItem");
    assert_eq!(ItemKind::Stone.class(), "BlockItem");
    assert_eq!(ItemKind::ZombieSpawnEgg.class(), "SpawnEggItem");
    assert!(ItemKind::ALL.iter().all(|kind| !kind.class().is_empty()));
}
