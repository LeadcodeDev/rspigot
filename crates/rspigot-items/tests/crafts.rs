use rspigot_data::{registries, tags};
use rspigot_items::{
    CookingKind, Craft, FilledMap, IronPickaxe, Item, ItemId, ItemKind, ItemSet, LavaBucket,
    MilkBucket, OakPlanks, Product, ProductComponent, StewEffect, SuspiciousStew, WaterBucket,
};

fn registry_item(name: &str) -> ItemId {
    let id = registries::static_index("minecraft:item", name).unwrap();
    ItemId(u16::try_from(id).unwrap())
}

fn is_only(set: &ItemSet, name: &str) -> bool {
    set.tag().is_none() && set.items() == [registry_item(name)]
}

fn all_crafts() -> Vec<(ItemKind, &'static Craft)> {
    ItemKind::ALL
        .into_iter()
        .flat_map(|kind| kind.crafts().iter().map(move |craft| (kind, craft)))
        .collect()
}

fn sets(craft: &Craft) -> Vec<ItemSet> {
    match *craft {
        Craft::Shaped { cells, .. } => cells.iter().flatten().copied().collect(),
        Craft::Shapeless { ingredients, .. } => ingredients.to_vec(),
        Craft::Transmute {
            input, material, ..
        } => vec![input, material],
        Craft::Dye { dye } => vec![dye],
        Craft::Imbue {
            source, material, ..
        } => vec![source, material],
        Craft::DecoratedPot {
            back,
            front,
            left,
            right,
        } => vec![back, front, left, right],
        Craft::BannerDuplicate | Craft::RepairItem => vec![],
        Craft::ShieldDecoration { banner } => vec![banner],
        Craft::MapExtending { map, material } => vec![map, material],
        Craft::FireworkStarFade { dye } => vec![dye],
        Craft::FireworkStar {
            dye,
            fuel,
            trail,
            twinkle,
            shapes,
        } => vec![
            dye,
            fuel,
            trail,
            twinkle,
            shapes.burst,
            shapes.creeper,
            shapes.large_ball,
            shapes.star,
        ],
        Craft::FireworkRocket {
            shell, star, fuel, ..
        } => vec![shell, star, fuel],
        Craft::BookCloning { source, material } => vec![source, material],
        Craft::Cooking { ingredient, .. } | Craft::Stonecutting { ingredient, .. } => {
            vec![ingredient]
        }
        Craft::SmithingTransform {
            template,
            base,
            addition,
        }
        | Craft::SmithingTrim {
            template,
            base,
            addition,
            ..
        } => vec![template, base, addition],
    }
}

fn count_of(predicate: impl Fn(&Craft) -> bool) -> usize {
    all_crafts()
        .into_iter()
        .filter(|(_, craft)| predicate(craft))
        .count()
}

#[test]
fn every_crafting_table_recipe_is_declared_on_an_item() {
    let damageable = ItemKind::ALL
        .into_iter()
        .filter(|kind| kind.max_damage().is_some())
        .count();
    assert_eq!(damageable, 84);
    assert_eq!(
        all_crafts()
            .into_iter()
            .filter(|(_, craft)| craft.uses_crafting_grid())
            .count(),
        1265 + 16 + damageable
    );
    assert_eq!(count_of(|c| matches!(c, Craft::RepairItem)), damageable);
    assert_eq!(count_of(|c| matches!(c, Craft::Shaped { .. })), 827);
    assert_eq!(count_of(|c| matches!(c, Craft::Shapeless { .. })), 375);
    assert_eq!(count_of(|c| matches!(c, Craft::Transmute { .. })), 33 + 16);
    assert_eq!(count_of(|c| matches!(c, Craft::Dye { .. })), 6);
    assert_eq!(count_of(|c| matches!(c, Craft::Imbue { .. })), 1);
    assert_eq!(count_of(|c| matches!(c, Craft::DecoratedPot { .. })), 1);
    assert_eq!(count_of(|c| matches!(c, Craft::BannerDuplicate)), 16);
    assert_eq!(count_of(|c| matches!(c, Craft::ShieldDecoration { .. })), 1);
    assert_eq!(count_of(|c| matches!(c, Craft::MapExtending { .. })), 1);
    assert_eq!(count_of(|c| matches!(c, Craft::FireworkStarFade { .. })), 1);
    assert_eq!(count_of(|c| matches!(c, Craft::FireworkStar { .. })), 1);
    assert_eq!(count_of(|c| matches!(c, Craft::FireworkRocket { .. })), 1);
    assert_eq!(count_of(|c| matches!(c, Craft::BookCloning { .. })), 1);
}

fn cooking(kind: CookingKind) -> Vec<(ItemKind, f32, u16)> {
    all_crafts()
        .into_iter()
        .filter_map(|(owner, craft)| match *craft {
            Craft::Cooking {
                kind: candidate,
                experience,
                cooking_time,
                ..
            } if candidate == kind => Some((owner, experience, cooking_time)),
            _ => None,
        })
        .collect()
}

#[test]
fn every_cooking_recipe_is_declared_on_the_item_it_makes() {
    assert_eq!(cooking(CookingKind::Smelting).len(), 73);
    assert_eq!(cooking(CookingKind::Blasting).len(), 25);
    assert_eq!(cooking(CookingKind::Smoking).len(), 9);
    assert_eq!(cooking(CookingKind::Campfire).len(), 9);
    let potato = cooking(CookingKind::Smelting)
        .into_iter()
        .find(|(owner, ..)| *owner == ItemKind::BakedPotato)
        .unwrap();
    assert_eq!((potato.1, potato.2), (0.35, 200));
    let campfire = cooking(CookingKind::Campfire)
        .into_iter()
        .find(|(owner, ..)| *owner == ItemKind::BakedPotato)
        .unwrap();
    assert_eq!(campfire.2, 600);
    let coal = cooking(CookingKind::Blasting)
        .into_iter()
        .find(|(owner, ..)| *owner == ItemKind::Coal)
        .unwrap();
    assert_eq!((coal.1, coal.2), (0.1, 200));
}

#[test]
fn a_cooking_recipe_carries_the_id_it_has_in_the_datapack() {
    let ids: Vec<&str> = ItemKind::BakedPotato
        .crafts()
        .iter()
        .filter_map(|craft| match craft {
            Craft::Cooking { id, .. } => Some(*id),
            _ => None,
        })
        .collect();
    assert_eq!(
        ids,
        [
            "minecraft:baked_potato",
            "minecraft:baked_potato_from_campfire_cooking",
            "minecraft:baked_potato_from_smoking"
        ]
    );
}

#[test]
fn fuels_burn_for_the_same_number_of_items_in_every_furnace() {
    let burn = |kind: ItemKind| {
        kind.cooking_fuel()
            .map(|fuel| (fuel.burn_time(false), fuel.burn_time(true)))
    };
    assert_eq!(burn(ItemKind::Coal), Some((1600, 800)));
    assert_eq!(burn(ItemKind::Charcoal), Some((1600, 800)));
    assert_eq!(burn(ItemKind::LavaBucket), Some((20000, 10000)));
    assert_eq!(burn(ItemKind::CoalBlock), Some((16000, 8000)));
    assert_eq!(burn(ItemKind::BlazeRod), Some((2400, 1200)));
    assert_eq!(burn(ItemKind::DriedKelpBlock), Some((4001, 2000)));
    assert_eq!(burn(ItemKind::Stick), Some((100, 50)));
    assert_eq!(burn(ItemKind::WhiteCarpet), Some((67, 33)));
    assert_eq!(burn(ItemKind::OakPlanks), Some((300, 150)));
    assert_eq!(burn(ItemKind::OakSlab), Some((150, 75)));
    assert_eq!(burn(ItemKind::CrimsonPlanks), None);
    assert_eq!(burn(ItemKind::OakLeaves), None);
    assert_eq!(burn(ItemKind::Dirt), None);
    assert_eq!(
        ItemKind::ALL
            .into_iter()
            .filter(|kind| kind.cooking_fuel().is_some())
            .count(),
        347
    );
}

#[test]
fn stonecutting_and_smithing_recipes_are_declared_too() {
    let counts: Vec<u8> = all_crafts()
        .into_iter()
        .filter_map(|(_, craft)| match *craft {
            Craft::Stonecutting { count, .. } => Some(count),
            _ => None,
        })
        .collect();
    assert_eq!(counts.len(), 351);
    assert_eq!(counts.iter().filter(|count| **count == 2).count(), 100);
    assert_eq!(counts.iter().filter(|count| **count == 8).count(), 8);
    assert_eq!(
        count_of(|c| matches!(c, Craft::SmithingTransform { .. })),
        12
    );
    assert_eq!(count_of(|c| matches!(c, Craft::SmithingTrim { .. })), 18);
    let [
        Craft::SmithingTransform {
            template,
            base,
            addition,
        },
        ..,
    ] = ItemKind::NetheriteAxe.crafts()
    else {
        panic!("{:?}", ItemKind::NetheriteAxe.crafts());
    };
    assert_eq!(
        template.items(),
        [registry_item(
            "minecraft:netherite_upgrade_smithing_template"
        )]
    );
    assert_eq!(base.items(), [registry_item("minecraft:diamond_axe")]);
    assert_eq!(addition.tag(), Some("minecraft:netherite_tool_materials"));
}

#[test]
fn a_trim_recipe_is_declared_on_its_template_because_the_result_is_dynamic() {
    let [
        Craft::SmithingTrim {
            pattern,
            base,
            addition,
            ..
        },
    ] = ItemKind::BoltArmorTrimSmithingTemplate
        .crafts()
        .iter()
        .filter(|craft| matches!(craft, Craft::SmithingTrim { .. }))
        .copied()
        .collect::<Vec<_>>()[..]
    else {
        panic!("{:?}", ItemKind::BoltArmorTrimSmithingTemplate.crafts());
    };
    assert_eq!(pattern, "minecraft:bolt");
    assert_eq!(base.tag(), Some("minecraft:trimmable_armor"));
    assert_eq!(addition.tag(), Some("minecraft:trim_materials"));
}

#[test]
fn an_item_declares_the_crafts_that_produce_it() {
    let [
        Craft::Shapeless {
            ingredients: [logs],
            product,
        },
    ] = OakPlanks::CRAFTS
    else {
        panic!("{:?}", OakPlanks::CRAFTS);
    };
    assert_eq!(logs.tag(), Some("minecraft:oak_logs"));
    assert_eq!(*product, Product::of(4));
    assert_eq!(ItemKind::OakPlanks.crafts(), OakPlanks::CRAFTS);
    assert!(ItemKind::Bedrock.crafts().is_empty());
}

#[test]
fn a_shaped_craft_keeps_its_pattern() {
    let [
        Craft::Shaped {
            width,
            height,
            symmetrical,
            cells,
            product,
        },
        Craft::RepairItem,
    ] = IronPickaxe::CRAFTS
    else {
        panic!("{:?}", IronPickaxe::CRAFTS);
    };
    assert_eq!((*width, *height, *symmetrical), (3, 3, true));
    assert_eq!(*product, Product::of(1));
    assert_eq!(cells.len(), 9);
    let materials = match cells[0] {
        Some(ItemSet::Tag(tag)) => tag,
        other => panic!("{other:?}"),
    };
    assert_eq!(materials.name, "minecraft:iron_tool_materials");
    assert!(cells[..3].iter().all(|cell| *cell == cells[0]));
    assert_eq!(cells[3], None);
    assert!(is_only(&cells[4].unwrap(), "minecraft:stick"));
    assert_eq!(cells[5], None);
    assert!(is_only(&cells[7].unwrap(), "minecraft:stick"));
}

#[test]
fn shaped_patterns_are_trimmed_to_their_bounding_box() {
    let shaped: Vec<_> = all_crafts()
        .into_iter()
        .filter_map(|(kind, craft)| match *craft {
            Craft::Shaped {
                width,
                height,
                symmetrical,
                cells,
                ..
            } => Some((
                kind,
                usize::from(width),
                usize::from(height),
                symmetrical,
                cells,
            )),
            _ => None,
        })
        .collect();
    assert_eq!(shaped.len(), 827);
    for (kind, width, height, symmetrical, cells) in shaped {
        let name = kind.name();
        assert!(
            (1..=3).contains(&width) && (1..=3).contains(&height),
            "{name}"
        );
        assert_eq!(cells.len(), width * height, "{name}");
        let filled = |x: usize, y: usize| cells[y * width + x].is_some();
        assert!((0..width).any(|x| filled(x, 0)), "{name} top row");
        assert!(
            (0..width).any(|x| filled(x, height - 1)),
            "{name} bottom row"
        );
        assert!((0..height).any(|y| filled(0, y)), "{name} left column");
        assert!(
            (0..height).any(|y| filled(width - 1, y)),
            "{name} right column"
        );
        let mirrored = (0..height).all(|y| {
            (0..width / 2).all(|x| cells[y * width + x] == cells[y * width + width - 1 - x])
        });
        assert_eq!(symmetrical, mirrored, "{name}");
    }
}

#[test]
fn an_asymmetric_pattern_is_flagged() {
    let hoe = ItemKind::WoodenHoe
        .crafts()
        .iter()
        .find(|craft| matches!(craft, Craft::Shaped { .. }))
        .unwrap();
    let Craft::Shaped {
        width,
        height,
        symmetrical,
        ..
    } = *hoe
    else {
        panic!("{hoe:?}");
    };
    assert_eq!((width, height, symmetrical), (2, 3, false));
}

#[test]
fn ingredient_sets_resolve_to_real_items() {
    let tag_sets: usize = all_crafts()
        .into_iter()
        .flat_map(|(_, craft)| sets(craft))
        .map(|set| {
            let items = set.items();
            assert!(!items.is_empty(), "{set:?}");
            assert!(items.windows(2).all(|pair| pair[0] < pair[1]), "{set:?}");
            assert!(items.iter().all(|item| item.kind().is_some()), "{set:?}");
            match set.tag() {
                Some(name) => {
                    let expected: Vec<u16> = tags::tag("minecraft:item", name)
                        .unwrap_or_else(|| panic!("{name}"))
                        .iter()
                        .map(|id| u16::try_from(*id).unwrap())
                        .collect();
                    let actual: Vec<u16> = items.iter().map(|item| item.0).collect();
                    assert_eq!(actual, expected, "{name}");
                    1
                }
                None => 0,
            }
        })
        .sum();
    assert_eq!(tag_sets, 398);
}

#[test]
fn a_craft_can_leave_an_item_behind() {
    let bucket = registry_item("minecraft:bucket");
    let bottle = registry_item("minecraft:glass_bottle");
    assert_eq!(WaterBucket::CRAFTING_REMAINDER, Some(bucket));
    assert_eq!(LavaBucket::CRAFTING_REMAINDER, Some(bucket));
    assert_eq!(MilkBucket::CRAFTING_REMAINDER, Some(bucket));
    assert_eq!(ItemKind::HoneyBottle.crafting_remainder(), Some(bottle));
    assert_eq!(ItemKind::DragonBreath.crafting_remainder(), Some(bottle));
    assert_eq!(OakPlanks::CRAFTING_REMAINDER, None);
    assert_eq!(
        ItemKind::ALL
            .into_iter()
            .filter(|kind| kind.crafting_remainder().is_some())
            .count(),
        5
    );
}

#[test]
fn suspicious_stew_recipes_carry_their_effect() {
    assert_eq!(SuspiciousStew::CRAFTS.len(), 17);
    let fire = SuspiciousStew::CRAFTS
        .iter()
        .find_map(|craft| match craft {
            Craft::Shapeless { product, .. } => match product.components {
                [ProductComponent::SuspiciousStewEffects([effect])]
                    if effect.effect == "minecraft:fire_resistance" =>
                {
                    Some((product, effect))
                }
                _ => None,
            },
            _ => None,
        })
        .unwrap();
    assert_eq!(fire.0.count, 1);
    assert_eq!(
        *fire.1,
        StewEffect {
            effect: "minecraft:fire_resistance",
            duration: 60,
        }
    );
}

#[test]
fn keeping_the_input_item_is_declared_on_each_item_it_applies_to() {
    let cloning: Vec<_> = all_crafts()
        .into_iter()
        .filter_map(|(kind, craft)| match *craft {
            Craft::Transmute {
                input,
                material,
                min_material,
                max_material,
                add_material_count: true,
                count,
            } => Some((kind, input, material, min_material, max_material, count)),
            _ => None,
        })
        .collect();
    assert_eq!(cloning.len(), 17);
    for (kind, input, material, low, high, count) in cloning {
        assert!(input.tag().is_none() && input.items() == [kind.id()]);
        assert!(is_only(&material, "minecraft:map"));
        assert_eq!((low, high, count), (1, 8, 1));
    }
    assert!(
        FilledMap::CRAFTS
            .iter()
            .any(|craft| matches!(craft, Craft::MapExtending { .. }))
    );
}

#[test]
fn a_transmute_without_a_range_takes_one_material() {
    let craft = all_crafts()
        .into_iter()
        .find(|(kind, craft)| {
            *kind == ItemKind::BlackShulkerBox && matches!(craft, Craft::Transmute { .. })
        })
        .unwrap()
        .1;
    let Craft::Transmute {
        input,
        material,
        min_material,
        max_material,
        add_material_count,
        count,
    } = *craft
    else {
        unreachable!()
    };
    assert_eq!(input.tag(), Some("minecraft:shulker_boxes"));
    assert!(is_only(&material, "minecraft:black_dye"));
    assert_eq!((min_material, max_material, count), (1, 1, 1));
    assert!(!add_material_count);
}

#[test]
fn special_crafts_name_their_ingredients() {
    let Some(Craft::FireworkRocket {
        shell,
        star,
        fuel,
        count,
    }) = ItemKind::FireworkRocket
        .crafts()
        .iter()
        .find(|craft| matches!(craft, Craft::FireworkRocket { .. }))
    else {
        panic!("{:?}", ItemKind::FireworkRocket.crafts());
    };
    assert!(is_only(shell, "minecraft:paper"));
    assert!(is_only(star, "minecraft:firework_star"));
    assert!(is_only(fuel, "minecraft:gunpowder"));
    assert_eq!(*count, 3);
    let crafts = ItemKind::FireworkStar.crafts();
    let Some(Craft::FireworkStar { shapes, trail, .. }) = crafts
        .iter()
        .find(|craft| matches!(craft, Craft::FireworkStar { .. }))
    else {
        panic!("{crafts:?}");
    };
    assert!(
        crafts
            .iter()
            .any(|craft| matches!(craft, Craft::FireworkStarFade { .. }))
    );
    assert!(is_only(&shapes.star, "minecraft:gold_nugget"));
    assert!(is_only(trail, "minecraft:diamond"));
    assert!(matches!(
        ItemKind::TippedArrow.crafts(),
        [Craft::Imbue { count: 8, .. }]
    ));
}
