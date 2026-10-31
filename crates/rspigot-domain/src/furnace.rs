use std::collections::{BTreeMap, HashMap};
use std::num::NonZeroU8;
use std::sync::OnceLock;

use rspigot_items::{CookingKind, Craft, ItemKind};

use crate::item::{ItemStack, Slot, count, max_stack, same_item_same_components, with_count};

pub const INPUT_SLOT: usize = 0;
pub const FUEL_SLOT: usize = 1;
pub const RESULT_SLOT: usize = 2;

const CONTAINER_MAX_STACK: u32 = 99;
const DEFAULT_COOKING_TIME: i32 = 200;
const FAST_SPEED: f32 = 2.0;
const NORMAL_SPEED: f32 = 1.0;
const DECAY_PER_TICK: i32 = 2;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CookingRecipe {
    pub id: &'static str,
    pub result: ItemKind,
    pub experience: f32,
    pub cooking_time: u16,
}

struct Index {
    by_input: HashMap<(CookingKind, u16), CookingRecipe>,
    by_id: HashMap<&'static str, CookingRecipe>,
}

fn index() -> &'static Index {
    static INDEX: OnceLock<Index> = OnceLock::new();
    INDEX.get_or_init(|| {
        let mut by_input: HashMap<(CookingKind, u16), CookingRecipe> = HashMap::new();
        let mut by_id = HashMap::new();
        for owner in ItemKind::ALL {
            for craft in owner.crafts() {
                let Craft::Cooking {
                    id,
                    kind,
                    ingredient,
                    experience,
                    cooking_time,
                } = *craft
                else {
                    continue;
                };
                let recipe = CookingRecipe {
                    id,
                    result: owner,
                    experience,
                    cooking_time,
                };
                by_id.insert(id, recipe);
                for input in ingredient.items() {
                    let slot = by_input.entry((kind, input.0)).or_insert(recipe);
                    if recipe.id < slot.id {
                        *slot = recipe;
                    }
                }
            }
        }
        Index { by_input, by_id }
    })
}

pub fn recipe_for(kind: CookingKind, input: &ItemStack) -> Option<CookingRecipe> {
    index().by_input.get(&(kind, input.item.0)).copied()
}

pub fn recipe_by_id(id: &str) -> Option<CookingRecipe> {
    index().by_id.get(id).copied()
}

pub fn can_smelt(kind: CookingKind, stack: &ItemStack) -> bool {
    recipe_for(kind, stack).is_some()
}

pub fn is_fuel(stack: &ItemStack) -> bool {
    stack
        .item
        .kind()
        .is_some_and(|kind| kind.cooking_fuel().is_some())
}

pub fn is_empty_bucket(stack: &ItemStack) -> bool {
    stack.item == ItemKind::Bucket.id()
}

fn is_fast(kind: CookingKind) -> bool {
    matches!(kind, CookingKind::Blasting | CookingKind::Smoking)
}

pub fn speed_of(kind: CookingKind) -> f32 {
    if is_fast(kind) {
        FAST_SPEED
    } else {
        NORMAL_SPEED
    }
}

fn burn_time(kind: CookingKind, stack: &ItemStack) -> i32 {
    stack
        .item
        .kind()
        .and_then(ItemKind::cooking_fuel)
        .map_or(0, |fuel| i32::from(fuel.burn_time(is_fast(kind))))
}

fn total_cook_time(cooking_time: u16, speed: f32) -> i32 {
    if speed > 0.0 {
        f64::from(f32::from(cooking_time) / speed).ceil() as i32
    } else {
        i32::from(cooking_time)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Furnace {
    pub items: [Slot; 3],
    pub lit_time_remaining: i32,
    pub lit_total_time: i32,
    pub cooking_time_spent: i32,
    pub cooking_total_time: i32,
    pub speed_multiplier: f32,
    pub recipes_used: BTreeMap<String, u32>,
}

impl Default for Furnace {
    fn default() -> Self {
        Self {
            items: [None, None, None],
            lit_time_remaining: 0,
            lit_total_time: 0,
            cooking_time_spent: 0,
            cooking_total_time: 0,
            speed_multiplier: NORMAL_SPEED,
            recipes_used: BTreeMap::new(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Tick {
    pub changed: bool,
    pub was_lit: bool,
    pub is_lit: bool,
}

impl Furnace {
    pub fn is_lit(&self) -> bool {
        self.lit_time_remaining > 0
    }

    pub fn data(&self) -> [i16; 4] {
        [
            self.lit_time_remaining,
            self.lit_total_time,
            self.cooking_time_spent,
            self.cooking_total_time,
        ]
        .map(|value| i16::try_from(value).unwrap_or(i16::MAX))
    }

    fn total_for_input(&self, kind: CookingKind) -> i32 {
        self.items[INPUT_SLOT]
            .as_ref()
            .and_then(|input| recipe_for(kind, input))
            .map_or(DEFAULT_COOKING_TIME, |recipe| {
                total_cook_time(recipe.cooking_time, self.speed_multiplier)
            })
    }

    pub fn input_replaced(&mut self, kind: CookingKind) {
        self.cooking_total_time = self.total_for_input(kind);
        self.cooking_time_spent = 0;
    }

    fn can_burn(&self, result: &ItemStack) -> bool {
        let Some(existing) = &self.items[RESULT_SLOT] else {
            return true;
        };
        same_item_same_components(existing, result)
            && count(&self.items[RESULT_SLOT]) + u32::from(result.count.get())
                <= CONTAINER_MAX_STACK.min(max_stack(result))
    }

    fn burn(&mut self, result: &ItemStack) {
        match &self.items[RESULT_SLOT] {
            None => self.items[RESULT_SLOT] = Some(result.clone()),
            Some(existing) => {
                let total = count(&self.items[RESULT_SLOT]) + u32::from(result.count.get());
                self.items[RESULT_SLOT] = with_count(existing, total);
            }
        }
        let wet_sponge = self.items[INPUT_SLOT]
            .as_ref()
            .is_some_and(|input| input.item == ItemKind::WetSponge.id());
        let empty_bucket = self.items[FUEL_SLOT].as_ref().is_some_and(is_empty_bucket);
        if wet_sponge && empty_bucket {
            self.items[FUEL_SLOT] =
                Some(ItemStack::new(ItemKind::WaterBucket.id(), NonZeroU8::MIN));
        }
        if let Some(input) = self.items[INPUT_SLOT].take() {
            self.items[INPUT_SLOT] = with_count(&input, u32::from(input.count.get()) - 1);
        }
    }

    fn consume_fuel(&mut self) {
        let Some(fuel) = self.items[FUEL_SLOT].take() else {
            return;
        };
        let remainder = fuel
            .item
            .kind()
            .and_then(ItemKind::crafting_remainder)
            .map(|item| ItemStack::new(item, NonZeroU8::MIN));
        self.items[FUEL_SLOT] = with_count(&fuel, u32::from(fuel.count.get()) - 1).or(remainder);
    }

    pub fn tick(&mut self, kind: CookingKind) -> Tick {
        let mut changed = false;
        let was_lit = self.is_lit();
        let mut is_lit = was_lit;
        if was_lit {
            self.lit_time_remaining -= 1;
            is_lit = self.lit_time_remaining > 0;
        }
        let fuelled = self.items[FUEL_SLOT].is_some() && self.items[INPUT_SLOT].is_some();
        if is_lit || fuelled {
            match self.items[INPUT_SLOT].clone() {
                None => self.cooking_time_spent = 0,
                Some(input) => {
                    if let Some(recipe) = recipe_for(kind, &input) {
                        changed |= self.cook(kind, recipe, &mut is_lit);
                    }
                }
            }
        } else if self.cooking_time_spent > 0 {
            self.cooking_time_spent =
                (self.cooking_time_spent - DECAY_PER_TICK).clamp(0, self.cooking_total_time);
        }
        if was_lit != is_lit {
            changed = true;
        }
        Tick {
            changed,
            was_lit,
            is_lit,
        }
    }

    fn cook(&mut self, kind: CookingKind, recipe: CookingRecipe, is_lit: &mut bool) -> bool {
        let result = ItemStack::new(recipe.result.id(), NonZeroU8::MIN);
        if !self.can_burn(&result) {
            self.cooking_time_spent = 0;
            return false;
        }
        let mut changed = false;
        if !*is_lit {
            let burn = self.items[FUEL_SLOT]
                .as_ref()
                .map_or(0, |fuel| burn_time(kind, fuel));
            self.lit_time_remaining = burn;
            self.lit_total_time = burn;
            self.speed_multiplier = speed_of(kind);
            if self.cooking_total_time > 0 && self.cooking_time_spent < self.cooking_total_time {
                let ratio = self.cooking_time_spent as f32 / self.cooking_total_time as f32;
                self.cooking_total_time =
                    total_cook_time(recipe.cooking_time, self.speed_multiplier);
                self.cooking_time_spent = (ratio * self.cooking_total_time as f32).ceil() as i32;
            }
            if burn > 0 {
                self.consume_fuel();
                *is_lit = true;
                changed = true;
            }
        }
        if !*is_lit {
            self.cooking_time_spent = 0;
            return changed;
        }
        self.cooking_time_spent += 1;
        if self.cooking_time_spent >= self.cooking_total_time {
            self.cooking_time_spent = 0;
            self.cooking_total_time = total_cook_time(recipe.cooking_time, self.speed_multiplier);
            self.burn(&result);
            *self.recipes_used.entry(recipe.id.to_owned()).or_insert(0) += 1;
            changed = true;
        }
        changed
    }

    pub fn experience(&self, next_float: &mut impl FnMut() -> f32) -> Vec<i32> {
        self.recipes_used
            .iter()
            .filter_map(|(id, uses)| {
                let recipe = recipe_by_id(id)?;
                let owed = *uses as f32 * recipe.experience;
                let mut whole = owed.floor() as i32;
                let fraction = owed - owed.floor();
                if fraction != 0.0 && next_float() < fraction {
                    whole += 1;
                }
                (whole > 0).then_some(whole)
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn stack(kind: ItemKind, amount: u8) -> Slot {
        Some(ItemStack::new(kind.id(), NonZeroU8::new(amount).unwrap()))
    }

    fn furnace_of(kind: CookingKind, input: Slot, fuel: Slot) -> Furnace {
        let mut furnace = Furnace {
            items: [input, fuel, None],
            ..Furnace::default()
        };
        furnace.input_replaced(kind);
        furnace
    }

    fn furnace(input: Slot, fuel: Slot) -> Furnace {
        furnace_of(CookingKind::Smelting, input, fuel)
    }

    fn run(furnace: &mut Furnace, kind: CookingKind, ticks: usize) {
        for _ in 0..ticks {
            furnace.tick(kind);
        }
    }

    #[test]
    fn a_fuelled_furnace_lights_up_on_the_first_tick() {
        let mut furnace = furnace(stack(ItemKind::Potato, 1), stack(ItemKind::Coal, 2));
        let tick = furnace.tick(CookingKind::Smelting);
        assert_eq!(
            tick,
            Tick {
                changed: true,
                was_lit: false,
                is_lit: true
            }
        );
        assert_eq!(furnace.items[FUEL_SLOT], stack(ItemKind::Coal, 1));
        assert_eq!(furnace.lit_time_remaining, 1600);
        assert_eq!(furnace.lit_total_time, 1600);
        assert_eq!(furnace.cooking_time_spent, 1);
    }

    #[test]
    fn an_item_is_ready_after_the_cooking_time_and_the_ignition_tick_counts() {
        let mut furnace = furnace(stack(ItemKind::Potato, 1), stack(ItemKind::Coal, 1));
        run(&mut furnace, CookingKind::Smelting, 199);
        assert_eq!(furnace.items[RESULT_SLOT], None);
        furnace.tick(CookingKind::Smelting);
        assert_eq!(furnace.items[RESULT_SLOT], stack(ItemKind::BakedPotato, 1));
        assert_eq!(furnace.items[INPUT_SLOT], None);
        assert_eq!(furnace.cooking_time_spent, 0);
        assert_eq!(furnace.recipes_used.get("minecraft:baked_potato"), Some(&1));
    }

    #[test]
    fn a_piece_of_coal_smelts_eight_items() {
        let mut furnace = furnace(stack(ItemKind::Potato, 20), stack(ItemKind::Coal, 1));
        run(&mut furnace, CookingKind::Smelting, 1600);
        assert_eq!(furnace.items[RESULT_SLOT], stack(ItemKind::BakedPotato, 8));
        assert_eq!(furnace.items[INPUT_SLOT], stack(ItemKind::Potato, 12));
        assert!(furnace.items[FUEL_SLOT].is_none());
        assert!(furnace.is_lit());
        furnace.tick(CookingKind::Smelting);
        assert!(!furnace.is_lit());
        assert_eq!(furnace.items[RESULT_SLOT], stack(ItemKind::BakedPotato, 8));
    }

    #[test]
    fn the_blast_furnace_and_the_smoker_cook_twice_as_fast_on_half_the_burn_time() {
        let mut blast = furnace_of(
            CookingKind::Blasting,
            stack(ItemKind::RawIron, 20),
            stack(ItemKind::Coal, 1),
        );
        blast.tick(CookingKind::Blasting);
        assert_eq!(blast.lit_time_remaining, 800);
        assert_eq!(blast.speed_multiplier, 2.0);
        run(&mut blast, CookingKind::Blasting, 99);
        assert_eq!(blast.items[RESULT_SLOT], stack(ItemKind::IronIngot, 1));
        run(&mut blast, CookingKind::Blasting, 700);
        assert_eq!(blast.items[RESULT_SLOT], stack(ItemKind::IronIngot, 8));
        let mut smoker = furnace_of(
            CookingKind::Smoking,
            stack(ItemKind::Beef, 1),
            stack(ItemKind::Stick, 1),
        );
        smoker.tick(CookingKind::Smoking);
        assert_eq!(smoker.lit_time_remaining, 50);
        smoker.lit_time_remaining = 800;
        run(&mut smoker, CookingKind::Smoking, 99);
        assert_eq!(smoker.items[RESULT_SLOT], stack(ItemKind::CookedBeef, 1));
    }

    #[test]
    fn each_kind_only_cooks_its_own_recipes() {
        let mut smoker = furnace_of(
            CookingKind::Smoking,
            stack(ItemKind::RawIron, 1),
            stack(ItemKind::Coal, 1),
        );
        run(&mut smoker, CookingKind::Smoking, 300);
        assert_eq!(smoker.items[RESULT_SLOT], None);
        assert_eq!(smoker.items[FUEL_SLOT], stack(ItemKind::Coal, 1));
        assert!(!smoker.is_lit());
        let mut blast = furnace_of(
            CookingKind::Blasting,
            stack(ItemKind::Beef, 1),
            stack(ItemKind::Coal, 1),
        );
        run(&mut blast, CookingKind::Blasting, 300);
        assert_eq!(blast.items[RESULT_SLOT], None);
    }

    #[test]
    fn nothing_happens_without_fuel_or_without_a_burnable_fuel() {
        let mut dry = furnace(stack(ItemKind::Potato, 1), None);
        let tick = dry.tick(CookingKind::Smelting);
        assert!(!tick.changed && !tick.is_lit);
        let mut stones = furnace(stack(ItemKind::Potato, 1), stack(ItemKind::Dirt, 1));
        run(&mut stones, CookingKind::Smelting, 50);
        assert!(!stones.is_lit());
        assert_eq!(stones.cooking_time_spent, 0);
        assert_eq!(stones.items[FUEL_SLOT], stack(ItemKind::Dirt, 1));
    }

    #[test]
    fn progress_fades_by_two_per_tick_when_the_fire_is_out() {
        let mut furnace = furnace(stack(ItemKind::Potato, 1), stack(ItemKind::Stick, 1));
        furnace.tick(CookingKind::Smelting);
        assert_eq!(furnace.lit_time_remaining, 100);
        run(&mut furnace, CookingKind::Smelting, 99);
        assert!(furnace.is_lit());
        assert_eq!(furnace.cooking_time_spent, 100);
        furnace.tick(CookingKind::Smelting);
        assert!(!furnace.is_lit());
        assert_eq!(furnace.cooking_time_spent, 98);
        run(&mut furnace, CookingKind::Smelting, 100);
        assert_eq!(furnace.cooking_time_spent, 0);
    }

    #[test]
    fn a_full_or_different_output_stops_the_cooking_but_not_the_fire() {
        let mut blocked = furnace(stack(ItemKind::Potato, 1), stack(ItemKind::Coal, 1));
        blocked.items[RESULT_SLOT] = stack(ItemKind::Stick, 1);
        run(&mut blocked, CookingKind::Smelting, 10);
        assert_eq!(blocked.items[FUEL_SLOT], stack(ItemKind::Coal, 1));
        assert!(!blocked.is_lit());
        let mut full = furnace(stack(ItemKind::Potato, 1), stack(ItemKind::Coal, 1));
        full.items[RESULT_SLOT] = stack(ItemKind::BakedPotato, 64);
        run(&mut full, CookingKind::Smelting, 10);
        assert_eq!(full.items[INPUT_SLOT], stack(ItemKind::Potato, 1));
        assert_eq!(full.cooking_time_spent, 0);
        assert!(!full.is_lit());
        let mut burning = furnace(stack(ItemKind::Potato, 2), stack(ItemKind::Coal, 1));
        run(&mut burning, CookingKind::Smelting, 5);
        burning.items[RESULT_SLOT] = stack(ItemKind::BakedPotato, 64);
        run(&mut burning, CookingKind::Smelting, 5);
        assert!(burning.is_lit());
        assert_eq!(burning.cooking_time_spent, 0);
        assert_eq!(burning.lit_time_remaining, 1591);
    }

    #[test]
    fn the_result_stacks_up_to_its_own_limit() {
        let mut furnace = furnace(stack(ItemKind::Potato, 1), stack(ItemKind::Coal, 1));
        furnace.items[RESULT_SLOT] = stack(ItemKind::BakedPotato, 63);
        run(&mut furnace, CookingKind::Smelting, 200);
        assert_eq!(furnace.items[RESULT_SLOT], stack(ItemKind::BakedPotato, 64));
    }

    #[test]
    fn a_lava_bucket_leaves_a_bucket_behind() {
        let mut furnace = furnace(stack(ItemKind::Potato, 1), stack(ItemKind::LavaBucket, 1));
        furnace.tick(CookingKind::Smelting);
        assert_eq!(furnace.items[FUEL_SLOT], stack(ItemKind::Bucket, 1));
        assert_eq!(furnace.lit_time_remaining, 20000);
    }

    #[test]
    fn a_wet_sponge_dries_out_and_fills_an_empty_bucket() {
        let mut furnace = furnace(stack(ItemKind::WetSponge, 1), stack(ItemKind::Bucket, 1));
        furnace.tick(CookingKind::Smelting);
        assert!(!furnace.is_lit());
        furnace.items[FUEL_SLOT] = stack(ItemKind::Coal, 2);
        furnace.items[RESULT_SLOT] = None;
        run(&mut furnace, CookingKind::Smelting, 200);
        assert_eq!(furnace.items[RESULT_SLOT], stack(ItemKind::Sponge, 1));
        assert_eq!(furnace.items[FUEL_SLOT], stack(ItemKind::Coal, 1));
        let mut bucket = furnace_with_bucket_after_ignition();
        bucket.items[INPUT_SLOT] = stack(ItemKind::WetSponge, 1);
        bucket.items[FUEL_SLOT] = stack(ItemKind::Bucket, 1);
        bucket.cooking_time_spent = 199;
        bucket.cooking_total_time = 200;
        bucket.lit_time_remaining = 50;
        bucket.tick(CookingKind::Smelting);
        assert_eq!(bucket.items[FUEL_SLOT], stack(ItemKind::WaterBucket, 1));
        assert_eq!(bucket.items[RESULT_SLOT], stack(ItemKind::Sponge, 1));
    }

    fn furnace_with_bucket_after_ignition() -> Furnace {
        Furnace {
            lit_total_time: 100,
            ..Furnace::default()
        }
    }

    #[test]
    fn taking_the_input_away_resets_the_progress_and_the_fire_burns_out_alone() {
        let mut furnace = furnace(stack(ItemKind::Potato, 1), stack(ItemKind::Stick, 1));
        run(&mut furnace, CookingKind::Smelting, 50);
        furnace.items[INPUT_SLOT] = None;
        furnace.tick(CookingKind::Smelting);
        assert_eq!(furnace.cooking_time_spent, 0);
        assert!(furnace.is_lit());
        run(&mut furnace, CookingKind::Smelting, 60);
        assert!(!furnace.is_lit());
    }

    #[test]
    fn lighting_a_fuel_while_partly_cooked_rescales_the_progress() {
        let mut furnace = furnace_of(
            CookingKind::Blasting,
            stack(ItemKind::RawIron, 1),
            stack(ItemKind::Coal, 1),
        );
        furnace.cooking_time_spent = 101;
        furnace.cooking_total_time = 200;
        furnace.tick(CookingKind::Blasting);
        assert_eq!(furnace.cooking_total_time, 100);
        assert_eq!(furnace.cooking_time_spent, 52);
    }

    #[test]
    fn the_total_time_of_a_new_input_follows_the_recipe_and_the_speed() {
        let mut furnace = Furnace::default();
        furnace.items[INPUT_SLOT] = stack(ItemKind::RawIron, 1);
        furnace.speed_multiplier = 2.0;
        furnace.cooking_time_spent = 77;
        furnace.input_replaced(CookingKind::Blasting);
        assert_eq!(
            (furnace.cooking_time_spent, furnace.cooking_total_time),
            (0, 100)
        );
        furnace.items[INPUT_SLOT] = stack(ItemKind::Dirt, 1);
        furnace.input_replaced(CookingKind::Blasting);
        assert_eq!(furnace.cooking_total_time, 200);
        furnace.items[INPUT_SLOT] = None;
        furnace.input_replaced(CookingKind::Blasting);
        assert_eq!(furnace.cooking_total_time, 200);
    }

    #[test]
    fn container_data_reports_the_four_counters() {
        let mut furnace = furnace(stack(ItemKind::Potato, 1), stack(ItemKind::Coal, 1));
        run(&mut furnace, CookingKind::Smelting, 5);
        assert_eq!(furnace.data(), [1596, 1600, 5, 200]);
        furnace.lit_time_remaining = 20000;
        furnace.lit_total_time = 40000;
        assert_eq!(furnace.data()[0], 20000);
        assert_eq!(furnace.data()[1], i16::MAX);
    }

    #[test]
    fn experience_is_owed_per_recipe_with_a_chance_for_the_fraction() {
        let mut furnace = Furnace::default();
        furnace
            .recipes_used
            .insert("minecraft:iron_ingot_from_smelting_raw_iron".to_owned(), 3);
        furnace
            .recipes_used
            .insert("minecraft:baked_potato".to_owned(), 1);
        furnace
            .recipes_used
            .insert("minecraft:removed_recipe".to_owned(), 9);
        let lucky = furnace.experience(&mut || 0.05);
        assert_eq!(lucky, vec![1, 3]);
        let unlucky = furnace.experience(&mut || 0.5);
        assert_eq!(unlucky, vec![2]);
        let exact = {
            let mut furnace = Furnace::default();
            furnace.recipes_used.insert("minecraft:coal".to_owned(), 10);
            furnace
        };
        assert_eq!(exact.experience(&mut || 0.0), Vec::<i32>::new());
    }

    #[test]
    fn recipes_are_found_by_input_and_by_id() {
        let input = ItemStack::new(ItemKind::Cod.id(), NonZeroU8::MIN);
        let smelting = recipe_for(CookingKind::Smelting, &input).unwrap();
        assert_eq!(smelting.result, ItemKind::CookedCod);
        assert_eq!(smelting.cooking_time, 200);
        assert_eq!(recipe_by_id(smelting.id), Some(smelting));
        assert!(recipe_for(CookingKind::Campfire, &input).is_some());
        assert!(!can_smelt(
            CookingKind::Smelting,
            &ItemStack::new(ItemKind::Dirt.id(), NonZeroU8::MIN)
        ));
        assert!(is_fuel(&ItemStack::new(
            ItemKind::Coal.id(),
            NonZeroU8::MIN
        )));
        assert!(!is_fuel(&ItemStack::new(
            ItemKind::Dirt.id(),
            NonZeroU8::MIN
        )));
    }
}
