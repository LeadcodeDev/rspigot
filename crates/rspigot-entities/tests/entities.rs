use rspigot_data::{entities, registries};
use rspigot_entities::{
    Cow, EntityDimensions, EntityKind, EntityType, EntityTypeId, Item, MobCategory, Player, Zombie,
};
use rspigot_items::{ItemId, ItemKind};

#[test]
fn every_vanilla_type_is_generated_in_registry_order() {
    assert_eq!(EntityKind::ALL.len(), 161);
    for (index, kind) in EntityKind::ALL.into_iter().enumerate() {
        assert_eq!(usize::from(kind.id().0), index);
        assert_eq!(EntityKind::from_id(kind.id()), Some(kind));
        assert_eq!(EntityKind::from_name(kind.name()), Some(kind));
    }
    assert_eq!(EntityKind::from_id(EntityTypeId(161)), None);
}

#[test]
fn ids_match_the_registry_report() {
    let entries = registries::static_entries("minecraft:entity_type").unwrap();
    assert_eq!(entries.len(), EntityKind::ALL.len());
    for kind in EntityKind::ALL {
        assert_eq!(entries[usize::from(kind.id().0)], kind.name());
        assert_eq!(entities::id(kind.name()), Some(kind.id().0));
    }
}

#[test]
fn attribute_names_exist_in_the_attribute_registry() {
    for kind in EntityKind::ALL {
        for (name, _) in kind.default_attributes() {
            assert!(registries::static_index("minecraft:attribute", name).is_some());
        }
    }
}

#[test]
fn zombie_matches_vanilla() {
    assert_eq!(Zombie::NAME, "minecraft:zombie");
    assert_eq!(Zombie::ID, EntityTypeId(154));
    assert_eq!(Zombie::CLASS, "Zombie");
    assert_eq!(Zombie::CATEGORY, MobCategory::Monster);
    assert_eq!(Zombie::WIDTH, 0.6);
    assert_eq!(Zombie::HEIGHT, 1.95);
    assert_eq!(Zombie::EYE_HEIGHT, 1.74);
    assert_eq!(
        (
            Zombie::LIVING,
            Zombie::SUMMONABLE,
            Zombie::FIRE_IMMUNE,
            Zombie::CAN_SPAWN_FAR_FROM_PLAYER
        ),
        (true, true, false, false)
    );
    assert_eq!(Zombie::CLIENT_TRACKING_RANGE, 8);
    assert_eq!(Zombie::UPDATE_INTERVAL, Some(3));
    let kind = EntityKind::Zombie;
    assert_eq!(kind.category(), MobCategory::Monster);
    assert_eq!(
        kind.dimensions(),
        EntityDimensions {
            width: 0.6,
            height: 1.95,
            eye_height: 1.74
        }
    );
    assert_eq!(kind.default_attribute("max_health"), Some(20.0));
    assert_eq!(kind.default_attribute("minecraft:max_health"), Some(20.0));
    assert_eq!(
        kind.default_attribute("movement_speed"),
        Some(f64::from(0.23_f32))
    );
    assert_eq!(kind.default_attribute("attack_damage"), Some(3.0));
    assert_eq!(kind.default_attribute("follow_range"), Some(35.0));
    assert_eq!(kind.default_attribute("attack_speed"), None);
    assert_eq!(kind.default_attribute("other:max_health"), None);
    assert_eq!(kind.default_attributes().len(), 29);
    assert_eq!(kind.default_attributes(), Zombie::DEFAULT_ATTRIBUTES);
}

#[test]
fn cow_matches_vanilla() {
    let kind = EntityKind::Cow;
    assert_eq!(Cow::ID, EntityTypeId(30));
    assert_eq!(kind.category(), MobCategory::Creature);
    assert_eq!(
        kind.dimensions(),
        EntityDimensions {
            width: 0.9,
            height: 1.4,
            eye_height: 1.3
        }
    );
    assert_eq!(kind.default_attribute("max_health"), Some(10.0));
    assert!(kind.can_spawn_far_from_player());
    assert_eq!(kind.tracking_range(), 10);
}

#[test]
fn player_is_living_and_not_summonable() {
    let kind = EntityKind::Player;
    assert_eq!(Player::ID, EntityTypeId(159));
    assert!(kind.living());
    assert!(!kind.summonable());
    assert_eq!(kind.category(), MobCategory::Misc);
    assert_eq!(kind.tracking_range(), 32);
    assert_eq!(kind.update_interval(), Some(2));
    assert_eq!(kind.spawn_egg(), None);
    assert_eq!(kind.default_attribute("max_health"), Some(20.0));
    assert_eq!(kind.default_attributes().len(), 36);
    assert_eq!(kind.dimensions().eye_height, 1.62);
}

#[test]
fn item_is_misc_and_not_living() {
    let kind = EntityKind::Item;
    assert_eq!(Item::NAME, "minecraft:item");
    assert_eq!(Item::ID, EntityTypeId(72));
    assert_eq!(Item::CLASS, "ItemEntity");
    assert_eq!(kind.category(), MobCategory::Misc);
    assert!(!kind.living());
    assert_eq!(kind.tracking_range(), 6);
    assert_eq!(kind.update_interval(), Some(20));
    assert!(kind.default_attributes().is_empty());
    assert_eq!(kind.default_attribute("max_health"), None);
    assert_eq!(
        kind.dimensions(),
        EntityDimensions {
            width: 0.25,
            height: 0.25,
            eye_height: 0.2125
        }
    );
}

#[test]
fn types_without_update_interval() {
    assert_eq!(EntityKind::Painting.update_interval(), None);
    let without = EntityKind::ALL
        .iter()
        .filter(|kind| kind.update_interval().is_none())
        .count();
    assert_eq!(without, 8);
}

#[test]
fn lookups_by_name() {
    assert_eq!(EntityKind::from_name("zombie"), Some(EntityKind::Zombie));
    assert_eq!(
        EntityKind::from_name("minecraft:zombie"),
        Some(EntityKind::Zombie)
    );
    assert_eq!(EntityKind::from_name("other:zombie"), None);
    assert_eq!(EntityKind::from_name("nope"), None);
}

#[test]
fn spawn_eggs_resolve_to_their_entity() {
    let item_id = |name: &str| ItemKind::from_name(name).map(|kind| kind.id().0);
    let egg = item_id("minecraft:zombie_spawn_egg").unwrap();
    assert_eq!(egg, 1333);
    assert_eq!(EntityKind::from_spawn_egg(egg), Some(EntityKind::Zombie));
    assert_eq!(Zombie::SPAWN_EGG, Some(egg));
    assert_eq!(Cow::SPAWN_EGG, item_id("minecraft:cow_spawn_egg"));
    assert_eq!(
        EntityKind::from_spawn_egg(item_id("minecraft:stone").unwrap()),
        None
    );
    let mut count = 0;
    for kind in EntityKind::ALL {
        if let Some(item) = kind.spawn_egg() {
            count += 1;
            assert_eq!(EntityKind::from_spawn_egg(item), Some(kind));
            assert_eq!(
                ItemKind::from_id(ItemId(item)).map(ItemKind::name),
                Some(format!("{}_spawn_egg", kind.name()).as_str())
            );
        }
    }
    assert_eq!(count, 88);
}

#[test]
fn living_types_have_default_attributes() {
    let living = EntityKind::ALL.iter().filter(|kind| kind.living()).count();
    assert_eq!(living, 93);
    for kind in EntityKind::ALL {
        assert_eq!(kind.living(), !kind.default_attributes().is_empty());
        assert_eq!(
            kind.living(),
            kind.default_attribute("max_health").is_some()
        );
    }
}

#[test]
fn counts_per_category() {
    let count = |category: MobCategory| {
        EntityKind::ALL
            .iter()
            .filter(|kind| kind.category() == category)
            .count()
    };
    assert_eq!(count(MobCategory::Monster), 45);
    assert_eq!(count(MobCategory::Creature), 31);
    assert_eq!(count(MobCategory::Ambient), 1);
    assert_eq!(count(MobCategory::Axolotls), 1);
    assert_eq!(count(MobCategory::UndergroundWaterCreature), 1);
    assert_eq!(count(MobCategory::WaterCreature), 3);
    assert_eq!(count(MobCategory::WaterAmbient), 4);
    assert_eq!(count(MobCategory::Misc), 75);
}

#[test]
fn mob_categories_match_vanilla() {
    let names: Vec<_> = MobCategory::ALL
        .iter()
        .map(|category| category.name())
        .collect();
    assert_eq!(
        names,
        [
            "monster",
            "creature",
            "ambient",
            "axolotls",
            "underground_water_creature",
            "water_creature",
            "water_ambient",
            "misc"
        ]
    );
    let row = |category: MobCategory| {
        (
            category.max_instances_per_chunk(),
            category.is_friendly(),
            category.is_persistent(),
            category.despawn_distance(),
            category.no_despawn_distance(),
        )
    };
    assert_eq!(row(MobCategory::Monster), (Some(70), false, false, 128, 32));
    assert_eq!(row(MobCategory::Creature), (Some(10), true, true, 128, 32));
    assert_eq!(row(MobCategory::Ambient), (Some(15), true, false, 128, 32));
    assert_eq!(row(MobCategory::Axolotls), (Some(5), true, false, 128, 32));
    assert_eq!(
        row(MobCategory::UndergroundWaterCreature),
        (Some(5), true, false, 128, 32)
    );
    assert_eq!(
        row(MobCategory::WaterCreature),
        (Some(5), true, false, 128, 32)
    );
    assert_eq!(
        row(MobCategory::WaterAmbient),
        (Some(20), true, false, 64, 32)
    );
    assert_eq!(row(MobCategory::Misc), (None, true, true, 128, 32));
}

#[test]
fn fire_immune_and_dragon() {
    let kind = EntityKind::EnderDragon;
    assert!(kind.fire_immune());
    assert_eq!(kind.category(), MobCategory::Monster);
    assert_eq!(kind.dimensions().eye_height, 6.8);
    let fire_immune = EntityKind::ALL
        .iter()
        .filter(|kind| kind.fire_immune())
        .count();
    assert_eq!(fire_immune, 15);
}
