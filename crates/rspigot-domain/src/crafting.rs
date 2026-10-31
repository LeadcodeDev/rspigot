use std::num::NonZeroU8;
use std::sync::OnceLock;

use rspigot_items::{Craft, ItemKind, ItemSet, Product, ProductComponent};
use rspigot_nbt::{Compound, Tag};

use crate::item::{ItemStack, Slot, count, same_item_same_components, with_count};

const SUSPICIOUS_STEW_EFFECTS: &str = "minecraft:suspicious_stew_effects";

struct Input<'a> {
    width: usize,
    height: usize,
    cells: Vec<Option<&'a ItemStack>>,
    filled: usize,
}

impl<'a> Input<'a> {
    fn of(width: usize, height: usize, slots: &'a [Slot]) -> Option<Self> {
        let occupied = |x: usize, y: usize| slots.get(y * width + x).is_some_and(Option::is_some);
        let columns: Vec<usize> = (0..width)
            .filter(|&x| (0..height).any(|y| occupied(x, y)))
            .collect();
        let rows: Vec<usize> = (0..height)
            .filter(|&y| (0..width).any(|x| occupied(x, y)))
            .collect();
        let (left, right) = (*columns.first()?, *columns.last()?);
        let (top, bottom) = (*rows.first()?, *rows.last()?);
        let cells: Vec<Option<&ItemStack>> = (top..=bottom)
            .flat_map(|y| (left..=right).map(move |x| (x, y)))
            .map(|(x, y)| slots[y * width + x].as_ref())
            .collect();
        let filled = cells.iter().flatten().count();
        Some(Self {
            width: right - left + 1,
            height: bottom - top + 1,
            cells,
            filled,
        })
    }

    fn stacks(&self) -> Vec<&'a ItemStack> {
        self.cells.iter().flatten().copied().collect()
    }
}

fn recipes() -> &'static [(ItemKind, &'static Craft)] {
    static INDEX: OnceLock<Vec<(ItemKind, &'static Craft)>> = OnceLock::new();
    INDEX.get_or_init(|| {
        ItemKind::ALL
            .into_iter()
            .flat_map(|kind| kind.crafts().iter().map(move |craft| (kind, craft)))
            .collect()
    })
}

fn find(input: &Input) -> Option<(ItemKind, &'static Craft)> {
    recipes()
        .iter()
        .find(|(owner, craft)| matches(*owner, craft, input))
        .copied()
}

fn matches(owner: ItemKind, craft: &Craft, input: &Input) -> bool {
    match *craft {
        Craft::Shaped {
            width,
            height,
            symmetrical,
            cells,
            ..
        } => shaped_matches(
            usize::from(width),
            usize::from(height),
            symmetrical,
            cells,
            input,
        ),
        Craft::Shapeless { ingredients, .. } => shapeless_matches(ingredients, input),
        Craft::Transmute {
            input: base,
            material,
            min_material,
            max_material,
            add_material_count,
            count,
        } => transmute_matches(
            owner,
            base,
            material,
            usize::from(min_material)..=usize::from(max_material),
            add_material_count,
            usize::from(count),
            input,
        ),
        _ => false,
    }
}

fn shaped_matches(
    width: usize,
    height: usize,
    symmetrical: bool,
    cells: &[Option<ItemSet>],
    input: &Input,
) -> bool {
    if input.filled != cells.iter().flatten().count()
        || input.width != width
        || input.height != height
    {
        return false;
    }
    let lays_over = |mirrored: bool| {
        (0..height).all(|y| {
            (0..width).all(|x| {
                let column = if mirrored { width - 1 - x } else { x };
                match (cells[y * width + column], input.cells[y * width + x]) {
                    (None, None) => true,
                    (Some(set), Some(stack)) => set.contains(stack.item),
                    _ => false,
                }
            })
        })
    };
    (!symmetrical && lays_over(true)) || lays_over(false)
}

fn shapeless_matches(ingredients: &[ItemSet], input: &Input) -> bool {
    if input.filled != ingredients.len() {
        return false;
    }
    let stacks = input.stacks();
    let mut claimed: Vec<Option<usize>> = vec![None; stacks.len()];
    (0..ingredients.len()).all(|ingredient| {
        let mut seen = vec![false; stacks.len()];
        claim(ingredient, ingredients, &stacks, &mut claimed, &mut seen)
    })
}

fn claim(
    ingredient: usize,
    ingredients: &[ItemSet],
    stacks: &[&ItemStack],
    claimed: &mut [Option<usize>],
    seen: &mut [bool],
) -> bool {
    for (slot, stack) in stacks.iter().enumerate() {
        if seen[slot] || !ingredients[ingredient].contains(stack.item) {
            continue;
        }
        seen[slot] = true;
        let free = match claimed[slot] {
            None => true,
            Some(other) => claim(other, ingredients, stacks, claimed, seen),
        };
        if free {
            claimed[slot] = Some(ingredient);
            return true;
        }
    }
    false
}

struct Transmuted<'a> {
    base: &'a ItemStack,
    materials: usize,
}

fn transmuted<'a>(
    base: ItemSet,
    material: ItemSet,
    max_material: usize,
    input: &Input<'a>,
) -> Option<Transmuted<'a>> {
    let mut found = None;
    let mut materials = 0;
    for stack in input.stacks() {
        if base.contains(stack.item) {
            if found.is_some() {
                return None;
            }
            found = Some(stack);
        } else if material.contains(stack.item) {
            materials += 1;
            if materials > max_material {
                return None;
            }
        } else {
            return None;
        }
    }
    found.map(|base| Transmuted { base, materials })
}

fn transmute_matches(
    owner: ItemKind,
    base: ItemSet,
    material: ItemSet,
    range: std::ops::RangeInclusive<usize>,
    add_material_count: bool,
    count: usize,
    input: &Input,
) -> bool {
    if input.filled < range.start() + 1 || input.filled > range.end() + 1 {
        return false;
    }
    let Some(found) = transmuted(base, material, *range.end(), input) else {
        return false;
    };
    if !range.contains(&found.materials) {
        return false;
    }
    let size = if add_material_count {
        found.materials + count
    } else {
        count
    };
    size != 1 || found.base.item != owner.id()
}

fn product_stack(owner: ItemKind, product: &Product) -> Slot {
    let mut components = Compound::new();
    for component in product.components {
        match component {
            ProductComponent::SuspiciousStewEffects(effects) => {
                let list = effects
                    .iter()
                    .map(|effect| {
                        Tag::Compound(
                            Compound::new()
                                .with("id", effect.effect)
                                .with("duration", effect.duration),
                        )
                    })
                    .collect();
                components.insert(SUSPICIOUS_STEW_EFFECTS, Tag::List(list));
            }
        }
    }
    Some(ItemStack {
        item: owner.id(),
        count: NonZeroU8::new(product.count)?,
        components,
    })
}

fn assemble(owner: ItemKind, craft: &Craft, input: &Input) -> Slot {
    match *craft {
        Craft::Shaped { product, .. } | Craft::Shapeless { product, .. } => {
            product_stack(owner, &product)
        }
        Craft::Transmute {
            input: base,
            material,
            max_material,
            add_material_count,
            count,
            ..
        } => {
            let found = transmuted(base, material, usize::from(max_material), input)?;
            let extra = if add_material_count {
                u8::try_from(found.materials).ok()?
            } else {
                0
            };
            Some(ItemStack {
                item: owner.id(),
                count: NonZeroU8::new(count.checked_add(extra)?)?,
                components: found.base.components.clone(),
            })
        }
        _ => None,
    }
}

fn remainder_of(stack: &ItemStack) -> Slot {
    let item = stack.item.kind()?.crafting_remainder()?;
    Some(ItemStack::new(item, NonZeroU8::MIN))
}

pub fn result(width: usize, height: usize, slots: &[Slot]) -> Slot {
    let input = Input::of(width, height, slots)?;
    let (owner, craft) = find(&input)?;
    assemble(owner, craft, &input)
}

pub fn take_result(width: usize, height: usize, slots: &mut [Slot]) -> Vec<ItemStack> {
    let matched = Input::of(width, height, slots).and_then(|input| find(&input));
    let mut leftovers = Vec::new();
    if matched.is_none() {
        return leftovers;
    }
    for slot in slots.iter_mut() {
        let Some(stack) = slot.take() else {
            continue;
        };
        *slot = with_count(&stack, u32::from(stack.count.get()) - 1);
        let Some(remainder) = remainder_of(&stack) else {
            continue;
        };
        match slot {
            None => *slot = Some(remainder),
            Some(existing) if same_item_same_components(existing, &remainder) => {
                let total = count(slot) + count(&Some(remainder.clone()));
                *slot = with_count(&remainder, total);
            }
            Some(_) => leftovers.push(remainder),
        }
    }
    leftovers
}

#[cfg(test)]
mod tests {
    use rspigot_items::ItemId;

    use super::*;

    fn stack(kind: ItemKind, amount: u8) -> Slot {
        Some(ItemStack::new(kind.id(), NonZeroU8::new(amount).unwrap()))
    }

    fn grid(width: usize, height: usize, filled: &[(usize, ItemKind, u8)]) -> Vec<Slot> {
        let mut slots = vec![None; width * height];
        for &(index, kind, amount) in filled {
            slots[index] = stack(kind, amount);
        }
        slots
    }

    fn crafted(width: usize, height: usize, filled: &[(usize, ItemKind, u8)]) -> Slot {
        result(width, height, &grid(width, height, filled))
    }

    #[test]
    fn an_empty_grid_crafts_nothing() {
        assert_eq!(crafted(3, 3, &[]), None);
        assert_eq!(crafted(2, 2, &[]), None);
    }

    #[test]
    fn a_log_makes_four_planks_wherever_it_sits() {
        for index in 0..4 {
            assert_eq!(
                crafted(2, 2, &[(index, ItemKind::OakLog, 1)]),
                stack(ItemKind::OakPlanks, 4)
            );
        }
        assert_eq!(
            crafted(3, 3, &[(8, ItemKind::BirchLog, 5)]),
            stack(ItemKind::BirchPlanks, 4)
        );
    }

    #[test]
    fn shaped_recipes_ignore_where_the_pattern_sits() {
        let planks = |indices: &[usize]| -> Vec<(usize, ItemKind, u8)> {
            indices
                .iter()
                .map(|&index| (index, ItemKind::OakPlanks, 1))
                .collect()
        };
        assert_eq!(crafted(2, 2, &planks(&[0, 2])), stack(ItemKind::Stick, 4));
        assert_eq!(crafted(2, 2, &planks(&[1, 3])), stack(ItemKind::Stick, 4));
        assert_eq!(crafted(3, 3, &planks(&[4, 7])), stack(ItemKind::Stick, 4));
        assert_eq!(crafted(3, 3, &planks(&[1, 4])), stack(ItemKind::Stick, 4));
        assert_eq!(crafted(3, 3, &planks(&[0, 2])), None);
        assert_eq!(crafted(2, 2, &planks(&[0, 3])), None);
    }

    #[test]
    fn a_four_by_four_square_of_planks_makes_a_crafting_table_in_the_small_grid() {
        let planks: Vec<_> = (0..4).map(|i| (i, ItemKind::OakPlanks, 1)).collect();
        assert_eq!(crafted(2, 2, &planks), stack(ItemKind::CraftingTable, 1));
        assert_eq!(crafted(2, 2, &planks[..3]), None);
    }

    #[test]
    fn the_whole_recipe_must_be_present_and_nothing_else() {
        let pickaxe = [
            (0, ItemKind::IronIngot, 1),
            (1, ItemKind::IronIngot, 1),
            (2, ItemKind::IronIngot, 1),
            (4, ItemKind::Stick, 1),
            (7, ItemKind::Stick, 1),
        ];
        assert_eq!(crafted(3, 3, &pickaxe), stack(ItemKind::IronPickaxe, 1));
        assert_eq!(crafted(3, 3, &pickaxe[..4]), None);
        let mut extra = pickaxe.to_vec();
        extra.push((8, ItemKind::Dirt, 1));
        assert_eq!(crafted(3, 3, &extra), None);
        let mut wrong = pickaxe.to_vec();
        wrong[0].1 = ItemKind::GoldIngot;
        assert_eq!(crafted(3, 3, &wrong), None);
    }

    #[test]
    fn a_tag_ingredient_takes_any_member() {
        for (material, pickaxe) in [
            (ItemKind::IronIngot, ItemKind::IronPickaxe),
            (ItemKind::Diamond, ItemKind::DiamondPickaxe),
            (ItemKind::GoldIngot, ItemKind::GoldenPickaxe),
            (ItemKind::CopperIngot, ItemKind::CopperPickaxe),
        ] {
            let slots = [
                (0, material, 1),
                (1, material, 1),
                (2, material, 1),
                (4, ItemKind::Stick, 1),
                (7, ItemKind::Stick, 1),
            ];
            assert_eq!(crafted(3, 3, &slots), stack(pickaxe, 1), "{material:?}");
        }
        let mixed = [
            (0, ItemKind::OakPlanks, 1),
            (1, ItemKind::SprucePlanks, 1),
            (2, ItemKind::OakPlanks, 1),
            (3, ItemKind::BirchPlanks, 1),
        ];
        assert_eq!(crafted(2, 2, &mixed), stack(ItemKind::CraftingTable, 1));
    }

    #[test]
    fn a_pattern_that_is_not_symmetric_also_matches_mirrored() {
        let hoe = |a: usize, b: usize, c: usize, d: usize| {
            [
                (a, ItemKind::OakPlanks, 1),
                (b, ItemKind::OakPlanks, 1),
                (c, ItemKind::Stick, 1),
                (d, ItemKind::Stick, 1),
            ]
        };
        assert_eq!(
            crafted(3, 3, &hoe(0, 1, 4, 7)),
            stack(ItemKind::WoodenHoe, 1)
        );
        assert_eq!(
            crafted(3, 3, &hoe(1, 2, 4, 7)),
            stack(ItemKind::WoodenHoe, 1)
        );
        assert_eq!(
            crafted(3, 3, &hoe(1, 2, 5, 8)),
            stack(ItemKind::WoodenHoe, 1)
        );
        assert_eq!(
            crafted(3, 3, &hoe(0, 1, 3, 6)),
            stack(ItemKind::WoodenHoe, 1)
        );
        assert_eq!(crafted(3, 3, &hoe(0, 2, 4, 7)), None);
    }

    #[test]
    fn shapeless_recipes_ignore_order_but_not_content() {
        let dye = |a: ItemKind, b: ItemKind| crafted(3, 3, &[(1, a, 1), (8, b, 1)]);
        assert_eq!(
            dye(ItemKind::RedDye, ItemKind::YellowDye),
            stack(ItemKind::OrangeDye, 2)
        );
        assert_eq!(
            dye(ItemKind::YellowDye, ItemKind::RedDye),
            stack(ItemKind::OrangeDye, 2)
        );
        assert_eq!(dye(ItemKind::RedDye, ItemKind::RedDye), None);
        assert_eq!(
            crafted(
                3,
                3,
                &[
                    (0, ItemKind::RedDye, 1),
                    (1, ItemKind::YellowDye, 1),
                    (2, ItemKind::BoneMeal, 1)
                ]
            ),
            None
        );
    }

    #[test]
    fn shapeless_matching_needs_every_ingredient_to_get_its_own_stack() {
        let flower_and_sugar =
            |first: ItemKind, second: ItemKind| crafted(3, 3, &[(0, first, 1), (1, second, 1)]);
        assert_eq!(flower_and_sugar(ItemKind::Sugar, ItemKind::Sugar), None);
        assert_eq!(
            crafted(
                3,
                3,
                &[
                    (0, ItemKind::MilkBucket, 1),
                    (1, ItemKind::Sugar, 1),
                    (2, ItemKind::Egg, 1),
                    (3, ItemKind::Wheat, 1)
                ]
            ),
            None
        );
    }

    #[test]
    fn a_stack_that_fits_two_ingredients_is_given_to_the_one_that_needs_it() {
        static EITHER: [ItemId; 2] = [ItemId(1), ItemId(2)];
        static FIRST: [ItemId; 1] = [ItemId(1)];
        let ingredients = [ItemSet::Direct(&EITHER), ItemSet::Direct(&FIRST)];
        let stone = ItemStack::new(ItemId(1), NonZeroU8::MIN);
        let granite = ItemStack::new(ItemId(2), NonZeroU8::MIN);
        for slots in [
            vec![Some(stone.clone()), Some(granite.clone())],
            vec![Some(granite.clone()), Some(stone.clone())],
        ] {
            let input = Input::of(2, 1, &slots).unwrap();
            assert!(shapeless_matches(&ingredients, &input));
        }
        let slots = vec![Some(granite.clone()), Some(granite)];
        let input = Input::of(2, 1, &slots).unwrap();
        assert!(!shapeless_matches(&ingredients, &input));
    }

    #[test]
    fn a_wider_pattern_never_reads_past_a_narrower_grid() {
        let block: Vec<_> = (0..4).map(|i| (i, ItemKind::Leather, 1)).collect();
        assert_eq!(crafted(2, 2, &block), None);
        let boots = [
            (0, ItemKind::Leather, 1),
            (2, ItemKind::Leather, 1),
            (3, ItemKind::Leather, 1),
            (5, ItemKind::Leather, 1),
        ];
        assert_eq!(crafted(3, 2, &boots), stack(ItemKind::LeatherBoots, 1));
        let lopsided = [
            (0, ItemKind::OakPlanks, 1),
            (1, ItemKind::OakPlanks, 1),
            (2, ItemKind::OakPlanks, 1),
            (3, ItemKind::OakPlanks, 1),
        ];
        assert_eq!(crafted(3, 2, &lopsided), None);
    }

    #[test]
    fn the_stack_sizes_in_the_grid_do_not_matter() {
        assert_eq!(
            crafted(2, 2, &[(0, ItemKind::OakLog, 64)]),
            stack(ItemKind::OakPlanks, 4)
        );
    }

    #[test]
    fn a_transmute_recipe_keeps_the_components_of_its_input() {
        let mut shulker = ItemStack::new(ItemKind::ShulkerBox.id(), NonZeroU8::MIN);
        shulker.components.insert("minecraft:custom_name", "kept");
        let mut slots = vec![None; 9];
        slots[0] = Some(shulker);
        slots[1] = stack(ItemKind::BlackDye, 1);
        let dyed = result(3, 3, &slots).unwrap();
        assert_eq!(dyed.item, ItemKind::BlackShulkerBox.id());
        assert_eq!(dyed.count.get(), 1);
        assert_eq!(
            dyed.components.get_str("minecraft:custom_name"),
            Some("kept")
        );
    }

    #[test]
    fn a_transmute_into_the_item_it_already_is_does_nothing() {
        assert_eq!(
            crafted(
                3,
                3,
                &[
                    (0, ItemKind::BlackShulkerBox, 1),
                    (1, ItemKind::BlackDye, 1)
                ]
            ),
            None
        );
        assert!(
            crafted(
                3,
                3,
                &[
                    (0, ItemKind::BlackShulkerBox, 1),
                    (1, ItemKind::WhiteDye, 1)
                ]
            )
            .is_some()
        );
    }

    #[test]
    fn a_transmute_needs_one_input_and_only_materials_besides() {
        assert_eq!(
            crafted(
                3,
                3,
                &[
                    (0, ItemKind::ShulkerBox, 1),
                    (1, ItemKind::BlackDye, 1),
                    (2, ItemKind::BlackDye, 1)
                ]
            ),
            None
        );
        assert_eq!(
            crafted(
                3,
                3,
                &[
                    (0, ItemKind::ShulkerBox, 1),
                    (1, ItemKind::WhiteShulkerBox, 1),
                    (2, ItemKind::BlackDye, 1)
                ]
            ),
            None
        );
        assert_eq!(crafted(3, 3, &[(1, ItemKind::BlackDye, 1)]), None);
    }

    #[test]
    fn cloning_a_map_counts_every_empty_map_as_a_copy() {
        let mut filled = ItemStack::new(ItemKind::FilledMap.id(), NonZeroU8::MIN);
        filled.components.insert("minecraft:map_id", 7);
        let mut slots = vec![None; 9];
        slots[0] = Some(filled);
        for slot in &mut slots[1..4] {
            *slot = stack(ItemKind::Map, 1);
        }
        let copies = result(3, 3, &slots).unwrap();
        assert_eq!(copies.item, ItemKind::FilledMap.id());
        assert_eq!(copies.count.get(), 4);
        assert_eq!(copies.components.get_i32("minecraft:map_id"), Some(7));
        for slot in &mut slots[4..] {
            *slot = stack(ItemKind::Map, 1);
        }
        assert_eq!(result(3, 3, &slots).unwrap().count.get(), 9);
    }

    #[test]
    fn cloning_keeps_the_map_kind() {
        let slots = grid(
            3,
            3,
            &[(0, ItemKind::DesertVillageMap, 1), (1, ItemKind::Map, 1)],
        );
        let copy = result(3, 3, &slots).unwrap();
        assert_eq!(copy.item, ItemKind::DesertVillageMap.id());
        assert_eq!(copy.count.get(), 2);
    }

    #[test]
    fn a_suspicious_stew_carries_its_effect() {
        let stew = crafted(
            3,
            3,
            &[
                (0, ItemKind::BrownMushroom, 1),
                (1, ItemKind::RedMushroom, 1),
                (2, ItemKind::Bowl, 1),
                (3, ItemKind::Dandelion, 1),
            ],
        )
        .unwrap();
        assert_eq!(stew.item, ItemKind::SuspiciousStew.id());
        let effects = stew
            .components
            .get_list("minecraft:suspicious_stew_effects")
            .unwrap();
        assert_eq!(effects.len(), 1);
        let effect = effects[0].as_compound().unwrap();
        assert_eq!(effect.get_str("id"), Some("minecraft:saturation"));
        assert_eq!(effect.get_i32("duration"), Some(7));
    }

    #[test]
    fn taking_the_result_uses_one_of_each_ingredient() {
        let mut slots = grid(
            3,
            3,
            &[(0, ItemKind::OakPlanks, 5), (3, ItemKind::OakPlanks, 1)],
        );
        assert!(take_result(3, 3, &mut slots).is_empty());
        assert_eq!(slots[0], stack(ItemKind::OakPlanks, 4));
        assert_eq!(slots[3], None);
    }

    #[test]
    fn taking_the_result_leaves_the_crafting_remainder() {
        let mut slots = grid(
            3,
            3,
            &[
                (0, ItemKind::MilkBucket, 1),
                (1, ItemKind::MilkBucket, 1),
                (2, ItemKind::MilkBucket, 1),
                (3, ItemKind::Sugar, 1),
                (4, ItemKind::Egg, 1),
                (5, ItemKind::Sugar, 1),
                (6, ItemKind::Wheat, 1),
                (7, ItemKind::Wheat, 1),
                (8, ItemKind::Wheat, 1),
            ],
        );
        assert_eq!(result(3, 3, &slots), stack(ItemKind::Cake, 1));
        assert!(take_result(3, 3, &mut slots).is_empty());
        assert!(
            slots[..3]
                .iter()
                .all(|slot| *slot == stack(ItemKind::Bucket, 1))
        );
        assert!(slots[3..].iter().all(Option::is_none));
    }

    #[test]
    fn a_remainder_that_cannot_take_the_emptied_slot_is_handed_over() {
        let mut slots = grid(2, 2, &[(0, ItemKind::HoneyBottle, 4)]);
        assert_eq!(result(2, 2, &slots), stack(ItemKind::Sugar, 3));
        assert_eq!(
            take_result(2, 2, &mut slots),
            vec![ItemStack::new(ItemKind::GlassBottle.id(), NonZeroU8::MIN)]
        );
        assert_eq!(slots[0], stack(ItemKind::HoneyBottle, 3));
    }

    #[test]
    fn taking_nothing_when_nothing_matches() {
        let mut slots = grid(3, 3, &[(0, ItemKind::Dirt, 3)]);
        assert!(take_result(3, 3, &mut slots).is_empty());
        assert_eq!(slots[0], stack(ItemKind::Dirt, 3));
    }
}
