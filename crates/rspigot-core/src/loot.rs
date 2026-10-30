use std::num::NonZeroU8;

use rand::Rng;
use rspigot_data::items;
use rspigot_data::loot::{
    self, BlockPredicate, BonusFormula, Condition, EnchantmentPredicate, EntityTarget, Entry,
    EntryKind, Function, FunctionKind, IntProvider, ItemPredicate, Leaf, LootTable, Output, Pool,
};
use rspigot_domain::item::ItemStack;
use rspigot_domain::{BlockStateId, ItemId};

const BLOCK_REGISTRY: &str = "minecraft:block";
const ITEM_REGISTRY: &str = "minecraft:item";
const LUCK: f32 = 0.0;

pub struct LootContext<'a> {
    pub state: BlockStateId,
    pub tool: Option<&'a ItemStack>,
}

pub fn tool_is_correct(tool: Option<&ItemStack>, state: BlockStateId) -> bool {
    let Some(kind) = state.kind() else {
        return true;
    };
    !kind.settings().requires_correct_tool
        || tool
            .and_then(|stack| items::tool(stack.item.0))
            .is_some_and(|rules| rules.is_correct_for_drops(u32::from(kind.id().0)))
}

pub fn block_drops<R: Rng + ?Sized>(context: &LootContext<'_>, rng: &mut R) -> Vec<ItemStack> {
    context
        .state
        .kind()
        .and_then(|kind| loot::block_table(kind.id().0))
        .map_or_else(Vec::new, |table| table_drops(table, context, rng))
}

fn table_drops<R: Rng + ?Sized>(
    table: &LootTable,
    context: &LootContext<'_>,
    rng: &mut R,
) -> Vec<ItemStack> {
    let mut roller = Roller {
        context,
        rng,
        drops: Vec::new(),
    };
    for pool in table.pools {
        roller.pool(pool, table.modifier);
    }
    roller.drops
}

fn held_enchantments(_tool: Option<&ItemStack>) -> &'static [(&'static str, i32)] {
    &[]
}

fn enchantment_level(tool: Option<&ItemStack>, enchantment: &str) -> i32 {
    held_enchantments(tool)
        .iter()
        .find(|(name, _)| *name == enchantment)
        .map_or(0, |(_, level)| *level)
}

fn enchantment_matches(predicate: &EnchantmentPredicate, held: &[(&str, i32)]) -> bool {
    match (predicate.enchantments, predicate.levels) {
        (Some(names), levels) => names.iter().any(|name| {
            held.iter().any(|(held_name, level)| {
                held_name == name
                    && *level != 0
                    && levels.is_none_or(|range| range.contains(*level))
            })
        }),
        (None, Some(range)) => held.iter().any(|(_, level)| range.contains(*level)),
        (None, None) => !held.is_empty(),
    }
}

fn item_matches(predicate: &ItemPredicate, tool: Option<&ItemStack>) -> bool {
    let item = tool.map_or(ItemId::AIR, |stack| stack.item);
    predicate
        .items
        .is_none_or(|items| items.contains(ITEM_REGISTRY, u32::from(item.0)))
        && predicate
            .enchantments
            .iter()
            .all(|enchantment| enchantment_matches(enchantment, held_enchantments(tool)))
}

fn block_matches(predicate: &BlockPredicate, state: BlockStateId) -> bool {
    let Some(kind) = state.kind() else {
        return false;
    };
    predicate
        .blocks
        .is_none_or(|blocks| blocks.contains(BLOCK_REGISTRY, u32::from(kind.id().0)))
        && predicate
            .state
            .iter()
            .all(|(name, value)| state.property(name) == Some(*value))
}

fn leaf_weight(leaf: &Leaf) -> i32 {
    (leaf.weight as f32 + leaf.quality as f32 * LUCK)
        .floor()
        .max(0.0) as i32
}

#[derive(Clone, Copy)]
enum Produce {
    Item(u16),
    Items(&'static [u32]),
    Nothing,
}

struct Candidate {
    produce: Produce,
    weight: i32,
    modifiers: Vec<&'static [Function]>,
}

struct Roller<'r, 'a, R: ?Sized> {
    context: &'r LootContext<'a>,
    rng: &'r mut R,
    drops: Vec<ItemStack>,
}

impl<R: Rng + ?Sized> Roller<'_, '_, R> {
    fn pool(&mut self, pool: &Pool, table_modifier: &'static [Function]) {
        if !self.passes(pool.condition.as_ref()) {
            return;
        }
        let rolls = self.int(pool.rolls) + (pool.bonus_rolls * LUCK).floor() as i32;
        for _ in 0..rolls {
            self.roll(pool, table_modifier);
        }
    }

    fn roll(&mut self, pool: &Pool, table_modifier: &'static [Function]) {
        let mut candidates = Vec::new();
        for entry in pool.entries {
            self.expand(entry, &[], &mut candidates);
        }
        let total: i32 = candidates.iter().map(|candidate| candidate.weight).sum();
        if total <= 0 {
            return;
        }
        let chosen = if candidates.len() == 1 {
            candidates.first()
        } else {
            let mut pick = self.rng.random_range(0..total);
            candidates.iter().find(|candidate| {
                pick -= candidate.weight;
                pick < 0
            })
        };
        if let Some(candidate) = chosen {
            let tail = [pool.modifier, table_modifier];
            match candidate.produce {
                Produce::Item(item) => self.emit(item, &candidate.modifiers, &tail),
                Produce::Items(ids) => {
                    for id in ids {
                        if let Ok(item) = u16::try_from(*id) {
                            self.emit(item, &candidate.modifiers, &tail);
                        }
                    }
                }
                Produce::Nothing => {}
            }
        }
    }

    fn expand(
        &mut self,
        entry: &'static Entry,
        outer: &[&'static [Function]],
        out: &mut Vec<Candidate>,
    ) -> bool {
        if !self.passes(entry.condition.as_ref()) {
            return false;
        }
        let mut chain = Vec::with_capacity(outer.len() + 1);
        chain.push(entry.modifier);
        chain.extend_from_slice(outer);
        match entry.kind {
            EntryKind::Leaf(leaf) => {
                Self::leaf(&leaf, chain, out);
                true
            }
            EntryKind::Alternatives(children) => {
                children.iter().any(|child| self.expand(child, &chain, out))
            }
            EntryKind::Sequence(children) => {
                children.iter().all(|child| self.expand(child, &chain, out))
            }
            EntryKind::Group(children) => {
                for child in children {
                    self.expand(child, &chain, out);
                }
                true
            }
        }
    }

    fn leaf(leaf: &Leaf, modifiers: Vec<&'static [Function]>, out: &mut Vec<Candidate>) {
        let weight = leaf_weight(leaf);
        if weight <= 0 {
            return;
        }
        let produce = match leaf.output {
            Output::Item(item) => Produce::Item(item),
            Output::Items {
                items,
                expand: true,
            } => {
                out.extend(
                    items
                        .ids(ITEM_REGISTRY)
                        .iter()
                        .filter_map(|id| u16::try_from(*id).ok())
                        .map(|item| Candidate {
                            produce: Produce::Item(item),
                            weight,
                            modifiers: modifiers.clone(),
                        }),
                );
                return;
            }
            Output::Items {
                items,
                expand: false,
            } => Produce::Items(items.ids(ITEM_REGISTRY)),
            Output::Empty | Output::Dynamic(_) => Produce::Nothing,
        };
        out.push(Candidate {
            produce,
            weight,
            modifiers,
        });
    }

    fn emit(&mut self, item: u16, modifiers: &[&'static [Function]], tail: &[&'static [Function]]) {
        let mut count = 1;
        for function in modifiers
            .iter()
            .chain(tail)
            .flat_map(|functions| functions.iter())
        {
            count = self.apply(function, count);
        }
        self.split(item, count);
    }

    fn split(&mut self, item: u16, count: i32) {
        if item == ItemId::AIR.0 {
            return;
        }
        let max = i32::from(items::max_stack_size(item)).max(1);
        let mut remaining = count;
        while remaining > 0 {
            let size = remaining.min(max);
            remaining -= size;
            if let Some(size) = u8::try_from(size).ok().and_then(NonZeroU8::new) {
                self.drops.push(ItemStack::new(ItemId(item), size));
            }
        }
    }

    fn apply(&mut self, function: &Function, count: i32) -> i32 {
        if !self.passes(function.condition.as_ref()) {
            return count;
        }
        let current = count.max(0);
        match function.kind {
            FunctionKind::SetCount { count: amount, add } => {
                let base = if add { current } else { 0 };
                base + self.int(amount)
            }
            FunctionKind::ApplyBonus {
                enchantment,
                formula,
            } => {
                let level = enchantment_level(self.context.tool, enchantment);
                self.bonus(formula, current, level)
            }
            FunctionKind::LimitCount { min, max } => {
                let low = min.map(|provider| self.int(provider));
                let high = max.map(|provider| self.int(provider));
                let raised = low.map_or(current, |low| current.max(low));
                high.map_or(raised, |high| raised.min(high))
            }
            FunctionKind::ExplosionDecay | FunctionKind::Components(_) => count,
        }
    }

    fn bonus(&mut self, formula: BonusFormula, count: i32, level: i32) -> i32 {
        match formula {
            BonusFormula::OreDrops => {
                if level > 0 {
                    let extra = (self.rng.random_range(0..level + 2) - 1).max(0);
                    count * (extra + 1)
                } else {
                    count
                }
            }
            BonusFormula::UniformBonusCount { bonus_multiplier } => {
                let bound = bonus_multiplier * level;
                if bound > 0 {
                    count + self.rng.random_range(0..=bound)
                } else {
                    count
                }
            }
            BonusFormula::BinomialWithBonusCount { extra, probability } => {
                count + self.successes(level + extra, probability)
            }
        }
    }

    fn successes(&mut self, trials: i32, probability: f32) -> i32 {
        let mut successes = 0;
        for _ in 0..trials {
            if self.rng.random::<f32>() < probability {
                successes += 1;
            }
        }
        successes
    }

    fn int(&mut self, provider: IntProvider) -> i32 {
        match provider {
            IntProvider::Constant(value) => value,
            IntProvider::Uniform { min, max } => {
                if min >= max {
                    min
                } else {
                    self.rng.random_range(min..=max)
                }
            }
            IntProvider::Binomial { n, p } => self.successes(n, p),
        }
    }

    fn passes(&mut self, condition: Option<&Condition>) -> bool {
        condition.is_none_or(|condition| self.test(condition))
    }

    fn test(&mut self, condition: &Condition) -> bool {
        match condition {
            Condition::SurvivesExplosion | Condition::LocationCheck { .. } => true,
            Condition::MatchBlock(predicate) => block_matches(predicate, self.context.state),
            Condition::MatchTool(predicate) => item_matches(predicate, self.context.tool),
            Condition::EntityProperties(target) => *target == EntityTarget::This,
            Condition::RandomChance(chance) => self.rng.random::<f32>() < *chance,
            Condition::TableBonus {
                enchantment,
                chances,
            } => {
                let level = enchantment_level(self.context.tool, enchantment);
                let index = usize::try_from(level)
                    .unwrap_or(0)
                    .min(chances.len().saturating_sub(1));
                chances
                    .get(index)
                    .is_some_and(|chance| self.rng.random::<f32>() < *chance)
            }
            Condition::Inverted(term) => !self.test(term),
            Condition::AnyOf(terms) => terms.iter().any(|term| self.test(term)),
            Condition::AllOf(terms) => terms.iter().all(|term| self.test(term)),
            Condition::Reference(name) => {
                loot::predicate(name).is_some_and(|condition| self.test(condition))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;
    use std::num::NonZeroU8;

    use rand::SeedableRng;
    use rand::rngs::StdRng;
    use rspigot_blocks::BlockKind;
    use rspigot_data::items;
    use rspigot_data::loot::{
        Condition, Entry, EntryKind, Function, FunctionKind, IntProvider, Leaf, Output, Pool,
    };
    use rspigot_data::tags::{self, HolderSet};
    use rspigot_domain::ItemId;

    use super::*;

    const RUNS: u64 = 10_000;

    fn item(name: &str) -> u16 {
        items::id(name).unwrap()
    }

    fn stack(name: &str) -> ItemStack {
        ItemStack::new(ItemId(item(name)), NonZeroU8::MIN)
    }

    fn kind(name: &str) -> BlockKind {
        BlockKind::from_name(name).unwrap()
    }

    fn block(name: &str) -> BlockStateId {
        kind(name).default_state()
    }

    fn state(name: &str, properties: &[(&str, &str)]) -> BlockStateId {
        kind(name).state_with(properties).unwrap()
    }

    fn summary(drops: &[ItemStack]) -> Vec<(&'static str, u8)> {
        drops
            .iter()
            .map(|stack| (items::name(stack.item.0).unwrap(), stack.count.get()))
            .collect()
    }

    fn drops(state: BlockStateId, tool: Option<&ItemStack>, seed: u64) -> Vec<(&'static str, u8)> {
        let context = LootContext { state, tool };
        summary(&block_drops(&context, &mut StdRng::seed_from_u64(seed)))
    }

    fn tally(state: BlockStateId, tool: Option<&ItemStack>) -> BTreeMap<(&'static str, u8), u32> {
        let mut rng = StdRng::seed_from_u64(7);
        let context = LootContext { state, tool };
        let mut counts = BTreeMap::new();
        for _ in 0..RUNS {
            for drop in summary(&block_drops(&context, &mut rng)) {
                *counts.entry(drop).or_insert(0) += 1;
            }
        }
        counts
    }

    fn occurrences(counts: &BTreeMap<(&'static str, u8), u32>, name: &str) -> u32 {
        counts
            .iter()
            .filter(|((item, _), _)| *item == name)
            .map(|(_, count)| count)
            .sum()
    }

    fn leaf(output: Output, weight: i32) -> Entry {
        Entry {
            condition: None,
            modifier: &[],
            kind: EntryKind::Leaf(Leaf {
                weight,
                quality: 0,
                output,
            }),
        }
    }

    fn table(entries: Vec<Entry>, modifier: Vec<Function>) -> LootTable {
        LootTable {
            pools: Vec::leak(vec![Pool {
                rolls: IntProvider::Constant(1),
                bonus_rolls: 0.0,
                condition: None,
                modifier: Vec::leak(modifier),
                entries: Vec::leak(entries),
            }]),
            modifier: &[],
        }
    }

    fn set_count(count: i32) -> Function {
        Function {
            condition: None,
            kind: FunctionKind::SetCount {
                count: IntProvider::Constant(count),
                add: false,
            },
        }
    }

    fn run(table: &LootTable, seed: u64) -> Vec<(&'static str, u8)> {
        let context = LootContext {
            state: block("minecraft:stone"),
            tool: None,
        };
        summary(&table_drops(
            table,
            &context,
            &mut StdRng::seed_from_u64(seed),
        ))
    }

    #[test]
    fn stone_with_stone_pickaxe_drops_cobblestone() {
        let pickaxe = stack("minecraft:stone_pickaxe");
        let stone = block("minecraft:stone");
        assert!(tool_is_correct(Some(&pickaxe), stone));
        assert_eq!(
            drops(stone, Some(&pickaxe), 1),
            [("minecraft:cobblestone", 1)]
        );
    }

    #[test]
    fn stone_by_hand_is_not_the_correct_tool() {
        assert!(!tool_is_correct(None, block("minecraft:stone")));
        assert!(!tool_is_correct(
            Some(&stack("minecraft:stick")),
            block("minecraft:stone")
        ));
        assert!(!tool_is_correct(
            Some(&stack("minecraft:iron_shovel")),
            block("minecraft:stone")
        ));
    }

    #[test]
    fn dirt_by_hand_drops_dirt() {
        let dirt = block("minecraft:dirt");
        assert!(tool_is_correct(None, dirt));
        assert_eq!(drops(dirt, None, 2), [("minecraft:dirt", 1)]);
    }

    #[test]
    fn diamond_ore_needs_iron_pickaxe() {
        let ore = block("minecraft:diamond_ore");
        let iron = stack("minecraft:iron_pickaxe");
        assert!(tool_is_correct(Some(&iron), ore));
        assert_eq!(drops(ore, Some(&iron), 3), [("minecraft:diamond", 1)]);
        assert!(!tool_is_correct(
            Some(&stack("minecraft:stone_pickaxe")),
            ore
        ));
        assert!(!tool_is_correct(None, ore));
    }

    #[test]
    fn oak_leaves_drop_saplings_sticks_and_apples() {
        let counts = tally(block("minecraft:oak_leaves"), None);
        assert!(counts.keys().all(|(name, count)| match *name {
            "minecraft:oak_sapling" | "minecraft:apple" => *count == 1,
            "minecraft:stick" => (1..=2).contains(count),
            _ => false,
        }));
        let saplings = occurrences(&counts, "minecraft:oak_sapling");
        let sticks = occurrences(&counts, "minecraft:stick");
        let apples = occurrences(&counts, "minecraft:apple");
        assert!((400..=600).contains(&saplings));
        assert!((130..=280).contains(&sticks));
        assert!((20..=90).contains(&apples));
        assert!(counts.contains_key(&("minecraft:stick", 1)));
        assert!(counts.contains_key(&("minecraft:stick", 2)));
    }

    #[test]
    fn oak_leaves_with_shears_drop_leaves() {
        let shears = stack("minecraft:shears");
        assert_eq!(
            drops(block("minecraft:oak_leaves"), Some(&shears), 4),
            [("minecraft:oak_leaves", 1)]
        );
    }

    #[test]
    fn double_oak_slab_drops_two_slabs() {
        let axe = stack("minecraft:wooden_axe");
        let double = state("minecraft:oak_slab", &[("type", "double")]);
        let bottom = state("minecraft:oak_slab", &[("type", "bottom")]);
        assert_eq!(drops(double, Some(&axe), 5), [("minecraft:oak_slab", 2)]);
        assert_eq!(drops(bottom, Some(&axe), 5), [("minecraft:oak_slab", 1)]);
    }

    #[test]
    fn ripe_wheat_drops_wheat_and_one_to_four_seeds() {
        let counts = tally(state("minecraft:wheat", &[("age", "7")]), None);
        assert_eq!(counts.get(&("minecraft:wheat", 1)), Some(&(RUNS as u32)));
        assert_eq!(occurrences(&counts, "minecraft:wheat_seeds"), RUNS as u32);
        for seeds in 1..=4 {
            assert!(counts.contains_key(&("minecraft:wheat_seeds", seeds)));
        }
        assert_eq!(counts.len(), 5);
    }

    #[test]
    fn young_wheat_drops_one_seed() {
        assert_eq!(
            drops(state("minecraft:wheat", &[("age", "0")]), None, 6),
            [("minecraft:wheat_seeds", 1)]
        );
    }

    #[test]
    fn glass_by_hand_drops_nothing() {
        let glass = block("minecraft:glass");
        assert!(tool_is_correct(None, glass));
        assert!(drops(glass, None, 7).is_empty());
    }

    #[test]
    fn grass_block_drops_dirt() {
        assert_eq!(
            drops(block("minecraft:grass_block"), None, 8),
            [("minecraft:dirt", 1)]
        );
    }

    #[test]
    fn tall_grass_lower_half_drops_seeds_one_time_in_eight() {
        let lower = state("minecraft:tall_grass", &[("half", "lower")]);
        let counts = tally(lower, None);
        assert_eq!(counts.len(), 1);
        let seeds = counts[&("minecraft:wheat_seeds", 1)];
        assert!((1100..=1400).contains(&seeds));
        let shears = stack("minecraft:shears");
        assert_eq!(
            drops(lower, Some(&shears), 9),
            [("minecraft:short_grass", 2)]
        );
    }

    #[test]
    fn air_has_no_drops() {
        assert!(drops(BlockStateId::AIR, None, 10).is_empty());
        assert!(drops(block("minecraft:water"), None, 10).is_empty());
        assert!(drops(BlockStateId(u16::MAX), None, 10).is_empty());
    }

    #[test]
    fn dirt_does_not_need_a_tool() {
        assert!(tool_is_correct(None, block("minecraft:dirt")));
    }

    #[test]
    fn cobweb_drops_cobweb_with_shears_and_string_with_a_sword() {
        let cobweb = block("minecraft:cobweb");
        let shears = stack("minecraft:shears");
        let sword = stack("minecraft:iron_sword");
        assert!(tool_is_correct(Some(&shears), cobweb));
        assert!(tool_is_correct(Some(&sword), cobweb));
        assert!(!tool_is_correct(None, cobweb));
        assert_eq!(drops(cobweb, Some(&shears), 11), [("minecraft:cobweb", 1)]);
        assert_eq!(drops(cobweb, Some(&sword), 11), [("minecraft:string", 1)]);
    }

    #[test]
    fn wall_torch_drops_a_torch() {
        assert_eq!(
            drops(block("minecraft:wall_torch"), None, 12),
            [("minecraft:torch", 1)]
        );
    }

    #[test]
    fn mushroom_block_count_is_clamped_at_zero() {
        let counts = tally(block("minecraft:brown_mushroom_block"), None);
        assert!(
            counts
                .keys()
                .all(|(name, count)| *name == "minecraft:brown_mushroom" && *count <= 2)
        );
        let total = occurrences(&counts, "minecraft:brown_mushroom");
        assert!((2000..=2450).contains(&total));
    }

    #[test]
    fn snow_layers_drop_matching_snowballs() {
        assert_eq!(
            drops(state("minecraft:snow", &[("layers", "5")]), None, 13),
            [("minecraft:snowball", 5)]
        );
    }

    #[test]
    fn redstone_ore_drops_four_or_five() {
        let pickaxe = stack("minecraft:iron_pickaxe");
        let counts = tally(block("minecraft:redstone_ore"), Some(&pickaxe));
        assert_eq!(
            counts.keys().copied().collect::<Vec<_>>(),
            [("minecraft:redstone", 4), ("minecraft:redstone", 5)]
        );
    }

    #[test]
    fn stacks_above_max_size_are_split() {
        let pearls = table(
            vec![leaf(Output::Item(item("minecraft:ender_pearl")), 1)],
            vec![set_count(40)],
        );
        assert_eq!(
            run(&pearls, 1),
            [
                ("minecraft:ender_pearl", 16),
                ("minecraft:ender_pearl", 16),
                ("minecraft:ender_pearl", 8)
            ]
        );
        let stone = table(
            vec![leaf(Output::Item(item("minecraft:stone")), 1)],
            vec![set_count(100)],
        );
        assert_eq!(
            run(&stone, 1),
            [("minecraft:stone", 64), ("minecraft:stone", 36)]
        );
        let none = table(
            vec![leaf(Output::Item(item("minecraft:stone")), 1)],
            vec![set_count(-3)],
        );
        assert!(run(&none, 1).is_empty());
    }

    #[test]
    fn weights_pick_entries_proportionally() {
        let weighted = table(
            vec![
                leaf(Output::Item(item("minecraft:stone")), 3),
                leaf(Output::Item(item("minecraft:dirt")), 1),
                leaf(Output::Item(item("minecraft:glass")), 0),
            ],
            vec![],
        );
        let mut stone = 0;
        for seed in 0..4000 {
            match run(&weighted, seed).as_slice() {
                [("minecraft:stone", 1)] => stone += 1,
                [("minecraft:dirt", 1)] => {}
                other => panic!("{other:?}"),
            }
        }
        assert!((2800..=3200).contains(&stone));
    }

    #[test]
    fn empty_entries_compete_for_the_roll() {
        let half = table(
            vec![
                leaf(Output::Empty, 1),
                leaf(Output::Item(item("minecraft:stone")), 1),
            ],
            vec![],
        );
        let hits = (0..4000)
            .filter(|seed| !run(&half, *seed).is_empty())
            .count();
        assert!((1800..=2200).contains(&hits));
    }

    #[test]
    fn composite_entries_follow_vanilla_expansion() {
        let never = Entry {
            condition: Some(Condition::Inverted(&Condition::SurvivesExplosion)),
            ..leaf(Output::Item(item("minecraft:glass")), 1)
        };
        let children: &'static [Entry] = Vec::leak(vec![
            leaf(Output::Item(item("minecraft:stone")), 1),
            never,
            leaf(Output::Item(item("minecraft:dirt")), 1),
        ]);
        let composite = |kind: EntryKind| Entry {
            condition: None,
            modifier: &[],
            kind,
        };
        let seen = |kind: EntryKind| {
            let table = table(vec![composite(kind)], vec![]);
            let mut names: Vec<&str> = (0..200)
                .flat_map(|seed| run(&table, seed))
                .map(|(name, _)| name)
                .collect();
            names.sort_unstable();
            names.dedup();
            names
        };
        assert_eq!(seen(EntryKind::Alternatives(children)), ["minecraft:stone"]);
        assert_eq!(seen(EntryKind::Sequence(children)), ["minecraft:stone"]);
        assert_eq!(
            seen(EntryKind::Group(children)),
            ["minecraft:dirt", "minecraft:stone"]
        );
    }

    #[test]
    fn composite_modifiers_apply_after_child_modifiers() {
        let child = Entry {
            modifier: Vec::leak(vec![set_count(3)]),
            ..leaf(Output::Item(item("minecraft:stone")), 1)
        };
        let add_two = Function {
            condition: None,
            kind: FunctionKind::SetCount {
                count: IntProvider::Constant(2),
                add: true,
            },
        };
        let parent = Entry {
            condition: None,
            modifier: Vec::leak(vec![add_two]),
            kind: EntryKind::Alternatives(Vec::leak(vec![child])),
        };
        let limit = Function {
            condition: None,
            kind: FunctionKind::LimitCount {
                min: None,
                max: Some(IntProvider::Constant(4)),
            },
        };
        assert_eq!(
            run(&table(vec![parent], vec![limit]), 1),
            [("minecraft:stone", 4)]
        );
    }

    #[test]
    fn tag_entries_drop_all_or_one_item() {
        let wool = HolderSet::Tag("minecraft:wool");
        let size = tags::tag("minecraft:item", "minecraft:wool").unwrap().len();
        let all = table(
            vec![leaf(
                Output::Items {
                    items: wool,
                    expand: false,
                },
                1,
            )],
            vec![],
        );
        assert_eq!(run(&all, 1).len(), size);
        let one = table(
            vec![leaf(
                Output::Items {
                    items: wool,
                    expand: true,
                },
                1,
            )],
            vec![],
        );
        let picked = run(&one, 1);
        assert_eq!(picked.len(), 1);
        assert!(picked[0].0.ends_with("_wool"));
    }
}
