use rspigot_items::ItemKind;

use crate::entity::reward::max_damage;
use crate::item::{
    HOTBAR_START, Inventory, ItemStack, Slot, count, max_stack, menu_slot, same_item,
    same_item_same_components, split, stacks_match, with_count,
};
use crate::menu::{
    ClickInput, Clicker, HashedSlot, MenuEffect, OUTSIDE, RemoteUpdate, place_back_or_drop,
};

const CONTAINER_MAX_STACK: u32 = 99;
const OFFHAND_BUTTON: i32 = 40;
const MAIN_START: usize = 9;
const MAIN_SLOTS: usize = 27;
const HOTBAR_SLOTS: usize = 9;
const SHULKER_BOXES_TAG: &str = "minecraft:shulker_boxes";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MenuKind {
    Generic { rows: u8 },
    Generic3x3,
    Hopper,
    ShulkerBox,
    Crafting,
    Furnace,
    BlastFurnace,
    Smoker,
    Anvil,
    Smithing,
    Grindstone,
    Stonecutter,
    CartographyTable,
    Loom,
}

impl MenuKind {
    pub fn protocol_id(self) -> i32 {
        match self {
            Self::Generic { rows } => i32::from(rows.clamp(1, 6)) - 1,
            Self::Generic3x3 => 6,
            Self::ShulkerBox => 20,
            Self::Hopper => 16,
            Self::Crafting => 12,
            Self::Furnace => 14,
            Self::BlastFurnace => 10,
            Self::Smoker => 22,
            Self::Anvil => 8,
            Self::Smithing => 21,
            Self::Grindstone => 15,
            Self::Stonecutter => 24,
            Self::CartographyTable => 23,
            Self::Loom => 18,
        }
    }

    pub fn container_size(self) -> usize {
        match self {
            Self::Generic { rows } => usize::from(rows.clamp(1, 6)) * 9,
            Self::Generic3x3 => 9,
            Self::Hopper => 5,
            Self::ShulkerBox => 27,
            Self::Crafting => 10,
            Self::Furnace | Self::BlastFurnace | Self::Smoker => 3,
            Self::Grindstone | Self::CartographyTable => 3,
            Self::Anvil => 3,
            Self::Smithing | Self::Loom => 4,
            Self::Stonecutter => 2,
        }
    }

    pub fn data_count(self) -> usize {
        match self {
            Self::Furnace | Self::BlastFurnace | Self::Smoker => 4,
            Self::Anvil | Self::Smithing | Self::Stonecutter | Self::Loom => 1,
            _ => 0,
        }
    }

    fn station_inputs(self) -> Option<usize> {
        match self {
            Self::Anvil | Self::Grindstone | Self::CartographyTable => Some(2),
            Self::Smithing | Self::Loom => Some(3),
            Self::Stonecutter => Some(1),
            _ => None,
        }
    }

    pub fn is_furnace(self) -> bool {
        matches!(self, Self::Furnace | Self::BlastFurnace | Self::Smoker)
    }

    fn station_plan(
        self,
        inputs: usize,
        slot: usize,
        stack: &ItemStack,
        container: &[Slot],
    ) -> Vec<(usize, usize, bool)> {
        let total = self.slot_count();
        let main = inputs + 1;
        let hotbar = main + MAIN_SLOTS;
        if slot == inputs {
            return vec![(main, total, true)];
        }
        if slot < inputs {
            return vec![(main, total, false)];
        }
        let swap = || {
            if slot < hotbar {
                vec![(hotbar, total, false)]
            } else {
                vec![(main, hotbar, false)]
            }
        };
        let filled = |index: usize| container.get(index).is_some_and(Option::is_some);
        match self {
            Self::Anvil => vec![(0, inputs, false)],
            Self::Grindstone if filled(0) && filled(1) => swap(),
            Self::Grindstone => vec![(0, 2, false)],
            Self::CartographyTable if is_map(stack) => vec![(0, 1, false)],
            Self::CartographyTable if is_map_material(stack) => vec![(1, 2, false)],
            Self::Loom if is_banner(stack) => vec![(0, 1, false)],
            Self::Loom if is_loom_dye(stack) => vec![(1, 2, false)],
            Self::Loom if is_loom_pattern(stack) => vec![(2, 3, false)],
            _ => swap(),
        }
    }

    fn quick_move_plan(
        self,
        slot: usize,
        stack: &ItemStack,
        container: &[Slot],
    ) -> Vec<(usize, usize, bool)> {
        let total = self.slot_count();
        if let Some(inputs) = self.station_inputs() {
            return self.station_plan(inputs, slot, stack, container);
        }
        match self {
            Self::Crafting => match slot {
                0 => vec![(10, 46, true)],
                1..=9 => vec![(10, 46, false)],
                10..=36 => vec![(1, 10, false), (37, 46, false)],
                _ => vec![(1, 10, false), (10, 37, false)],
            },
            Self::Furnace | Self::BlastFurnace | Self::Smoker => match slot {
                2 => vec![(3, total, true)],
                0 | 1 => vec![(3, total, false)],
                3..=29 => vec![(30, total, false)],
                _ => vec![(3, 30, false)],
            },
            _ => {
                let size = self.container_size();
                if slot < size {
                    vec![(size, total, true)]
                } else {
                    vec![(0, size, false)]
                }
            }
        }
    }

    fn clears_on_close(self, slot: usize) -> bool {
        if let Some(inputs) = self.station_inputs() {
            return slot < inputs;
        }
        matches!(self, Self::Crafting) && (1..=9).contains(&slot)
    }

    pub fn slot_count(self) -> usize {
        self.container_size() + MAIN_SLOTS + HOTBAR_SLOTS
    }

    fn accepts(self, slot: usize, stack: &ItemStack) -> bool {
        if let Some(inputs) = self.station_inputs()
            && slot == inputs
        {
            return false;
        }
        match self {
            Self::Grindstone if slot < 2 => is_damageable(stack) || has_enchantments(stack),
            Self::CartographyTable if slot == 0 => is_map(stack),
            Self::CartographyTable if slot == 1 => is_map_material(stack),
            Self::Loom if slot == 0 => is_banner(stack),
            Self::Loom if slot == 1 => is_loom_dye(stack),
            Self::Loom if slot == 2 => is_loom_pattern(stack),
            Self::ShulkerBox if slot < self.container_size() => !is_shulker_box(stack),
            Self::Crafting => slot != 0,
            Self::Furnace | Self::BlastFurnace | Self::Smoker => slot != 2,
            _ => true,
        }
    }
}

fn in_item_tag(stack: &ItemStack, tag: &str) -> bool {
    rspigot_data::tags::tag("minecraft:item", tag)
        .is_some_and(|ids| ids.binary_search(&u32::from(stack.item.0)).is_ok())
}

fn is_shulker_box(stack: &ItemStack) -> bool {
    in_item_tag(stack, SHULKER_BOXES_TAG)
}

fn is_damageable(stack: &ItemStack) -> bool {
    max_damage(stack).is_some()
}

fn lists_enchantments(stack: &ItemStack, key: &str) -> bool {
    stack
        .components
        .get_compound(key)
        .is_some_and(|enchantments| {
            enchantments
                .get_compound("levels")
                .map_or(!enchantments.is_empty(), |levels| !levels.is_empty())
        })
}

fn has_enchantments(stack: &ItemStack) -> bool {
    lists_enchantments(stack, "minecraft:enchantments")
        || lists_enchantments(stack, "minecraft:stored_enchantments")
}

fn is_map(stack: &ItemStack) -> bool {
    stack.components.contains_key("minecraft:map_id")
}

fn is_map_material(stack: &ItemStack) -> bool {
    matches!(
        stack.item.kind(),
        Some(ItemKind::Paper | ItemKind::Map | ItemKind::GlassPane)
    )
}

fn is_banner(stack: &ItemStack) -> bool {
    in_item_tag(stack, "minecraft:banners")
}

fn is_loom_dye(stack: &ItemStack) -> bool {
    in_item_tag(stack, "minecraft:loom_dyes") || stack.components.contains_key("minecraft:dye")
}

fn is_loom_pattern(stack: &ItemStack) -> bool {
    in_item_tag(stack, "minecraft:loom_patterns")
        || stack
            .components
            .contains_key("minecraft:provides_banner_patterns")
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Cell {
    Container(usize),
    Player(usize),
}

#[derive(Debug, Clone, PartialEq)]
enum RemoteSlot {
    Known(Slot),
    Hashed(HashedSlot),
}

impl RemoteSlot {
    fn matches(&mut self, stack: &Slot) -> bool {
        match self {
            Self::Known(remote) => stacks_match(remote, stack),
            Self::Hashed(hash) if hash.matches(stack) => {
                *self = Self::Known(stack.clone());
                true
            }
            Self::Hashed(_) => false,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
struct QuickCraft {
    status: i32,
    kind: i32,
    slots: Vec<usize>,
}

impl QuickCraft {
    fn reset(&mut self) {
        self.status = 0;
        self.slots.clear();
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ContainerMenu {
    kind: MenuKind,
    carried: Slot,
    quick_craft: QuickCraft,
    remote: Vec<RemoteSlot>,
    remote_carried: RemoteSlot,
}

struct Session<'a> {
    kind: MenuKind,
    container: &'a mut Vec<Slot>,
    player: &'a mut Inventory,
    carried: &'a mut Slot,
    infinite: bool,
    effects: Vec<MenuEffect>,
}

fn quick_craft_place_count(slots: usize, kind: i32, stack: &ItemStack) -> u32 {
    match kind {
        0 => u32::from(stack.count.get()) / u32::try_from(slots).unwrap_or(u32::MAX).max(1),
        1 => 1,
        2 => max_stack(stack),
        _ => u32::from(stack.count.get()),
    }
}

impl Session<'_> {
    fn cell(&self, slot: usize) -> Option<Cell> {
        let size = self.kind.container_size();
        if slot < size {
            Some(Cell::Container(slot))
        } else if slot < size + MAIN_SLOTS {
            Some(Cell::Player(MAIN_START + slot - size))
        } else if slot < size + MAIN_SLOTS + HOTBAR_SLOTS {
            Some(Cell::Player(HOTBAR_START + slot - size - MAIN_SLOTS))
        } else {
            None
        }
    }

    fn index(&self, slot: i32) -> Option<usize> {
        usize::try_from(slot)
            .ok()
            .filter(|slot| *slot < self.kind.slot_count())
    }

    fn item(&self, slot: usize) -> Slot {
        match self.cell(slot) {
            Some(Cell::Container(index)) => self.container.get(index).cloned().flatten(),
            Some(Cell::Player(index)) => self.player.get(index).cloned(),
            None => None,
        }
    }

    fn put(&mut self, slot: usize, stack: Slot) {
        match self.cell(slot) {
            Some(Cell::Container(index)) => {
                if let Some(entry) = self.container.get_mut(index) {
                    *entry = stack;
                }
            }
            Some(Cell::Player(index)) => {
                self.player.set(index, stack);
            }
            None => {}
        }
    }

    fn remove(&mut self, slot: usize, amount: u32) -> Slot {
        if amount == 0 {
            return None;
        }
        let mut current = self.item(slot);
        let removed = split(&mut current, amount);
        self.put(slot, current);
        removed
    }

    fn may_place(&self, slot: usize, stack: &Slot) -> bool {
        stack
            .as_ref()
            .is_none_or(|stack| self.kind.accepts(slot, stack))
    }

    fn slot_max_for(&self, stack: &ItemStack) -> u32 {
        CONTAINER_MAX_STACK.min(max_stack(stack))
    }

    fn try_remove(&mut self, slot: usize, amount: u32, decrement: u32) -> Slot {
        self.remove(slot, amount.min(decrement))
    }

    fn safe_insert(&mut self, slot: usize, stack: Slot, increment: u32) -> Slot {
        let Some(incoming) = stack.clone() else {
            return stack;
        };
        if !self.may_place(slot, &stack) {
            return stack;
        }
        let current = self.item(slot);
        let room = i64::from(self.slot_max_for(&incoming)) - i64::from(count(&current));
        let placed = i64::from(increment.min(count(&stack))).min(room);
        let Ok(placed) = u32::try_from(placed) else {
            return stack;
        };
        if placed == 0 {
            return stack;
        }
        let mut remaining = stack;
        match &current {
            None => {
                let part = split(&mut remaining, placed);
                self.put(slot, part);
            }
            Some(existing) if same_item_same_components(existing, &incoming) => {
                split(&mut remaining, placed);
                self.put(slot, with_count(existing, count(&current) + placed));
            }
            Some(_) => {}
        }
        remaining
    }

    fn drop_stack(&mut self, stack: Slot, thrower: bool, predicted: bool) -> bool {
        match stack {
            Some(stack) => {
                self.effects.push(MenuEffect::Drop {
                    stack,
                    thrower,
                    predicted,
                });
                true
            }
            None => false,
        }
    }

    fn can_quick_replace(&self, slot: usize, stack: &ItemStack) -> bool {
        match self.item(slot) {
            None => true,
            Some(existing) => {
                same_item_same_components(stack, &existing)
                    && u32::from(existing.count.get()) <= max_stack(stack)
            }
        }
    }

    fn click(&mut self, craft: &mut QuickCraft, slot: i32, button: i32, input: ClickInput) {
        if input == ClickInput::QuickCraft {
            self.quick_craft(craft, slot, button);
            return;
        }
        if craft.status != 0 {
            craft.reset();
            return;
        }
        match input {
            ClickInput::Pickup | ClickInput::QuickMove if button == 0 || button == 1 => {
                self.pickup(slot, button == 0, input == ClickInput::QuickMove);
            }
            ClickInput::Swap if (0..9).contains(&button) || button == OFFHAND_BUTTON => {
                self.swap(slot, button);
            }
            ClickInput::Clone => self.clone_slot(slot),
            ClickInput::Throw => self.throw(slot, button),
            ClickInput::PickupAll => self.pickup_all(slot, button),
            _ => {}
        }
    }

    fn quick_craft(&mut self, craft: &mut QuickCraft, slot: i32, button: i32) {
        let previous = craft.status;
        craft.status = button & 3;
        if (previous != 1 || craft.status != 2) && previous != craft.status {
            craft.reset();
            return;
        }
        let Some(carried) = self.carried.clone() else {
            craft.reset();
            return;
        };
        match craft.status {
            0 => {
                craft.kind = (button >> 2) & 3;
                if craft.kind == 0 || craft.kind == 1 || (craft.kind == 2 && self.infinite) {
                    craft.status = 1;
                    craft.slots.clear();
                } else {
                    craft.reset();
                }
            }
            1 => {
                let Some(target) = self.index(slot) else {
                    return;
                };
                let wide = craft.kind == 2
                    || u32::from(carried.count.get())
                        > u32::try_from(craft.slots.len()).unwrap_or(u32::MAX);
                if self.can_quick_replace(target, &carried)
                    && self.may_place(target, &Some(carried.clone()))
                    && wide
                    && !craft.slots.contains(&target)
                {
                    craft.slots.push(target);
                }
            }
            2 => {
                if !craft.slots.is_empty() {
                    self.finish_quick_craft(craft, &carried);
                }
                craft.reset();
            }
            _ => craft.reset(),
        }
    }

    fn finish_quick_craft(&mut self, craft: &QuickCraft, copy: &ItemStack) {
        let spread = craft.slots.len();
        let mut remaining = i64::from(copy.count.get());
        let mut placements = Vec::new();
        for &target in &craft.slots {
            let wide = craft.kind == 2
                || u32::from(copy.count.get()) >= u32::try_from(spread).unwrap_or(u32::MAX);
            if !self.can_quick_replace(target, copy)
                || !self.may_place(target, &Some(copy.clone()))
                || !wide
            {
                continue;
            }
            let existing = count(&self.item(target));
            let limit = max_stack(copy).min(self.slot_max_for(copy));
            let placed = (quick_craft_place_count(spread, craft.kind, copy) + existing).min(limit);
            remaining -= i64::from(placed) - i64::from(existing);
            placements.push((target, with_count(copy, placed)));
        }
        *self.carried = with_count(copy, u32::try_from(remaining.max(0)).unwrap_or(0));
        for (target, stack) in placements {
            self.put(target, stack);
        }
        self.effects.push(MenuEffect::FullResync);
    }

    fn pickup(&mut self, slot: i32, primary: bool, quick_move: bool) {
        if slot == i32::from(OUTSIDE) {
            if self.carried.is_some() {
                let thrown = if primary {
                    self.carried.take()
                } else {
                    split(self.carried, 1)
                };
                self.drop_stack(thrown, true, true);
            }
            return;
        }
        let Some(target) = self.index(slot) else {
            return;
        };
        if quick_move {
            let mut moved = self.quick_move_stack(target);
            while moved.is_some() && same_item(&self.item(target), &moved) {
                moved = self.quick_move_stack(target);
            }
            return;
        }
        let in_slot = self.item(target);
        let carried = self.carried.clone();
        match (&in_slot, &carried) {
            (None, Some(_)) => {
                let amount = if primary { count(&carried) } else { 1 };
                *self.carried = self.safe_insert(target, carried, amount);
            }
            (None, None) => {}
            (Some(existing), _) => match &carried {
                None => {
                    let amount = if primary {
                        count(&in_slot)
                    } else {
                        count(&in_slot).div_ceil(2)
                    };
                    if let Some(taken) = self.try_remove(target, amount, u32::MAX) {
                        *self.carried = Some(taken);
                    }
                }
                Some(held) if self.may_place(target, &carried) => {
                    if same_item_same_components(existing, held) {
                        let amount = if primary { count(&carried) } else { 1 };
                        *self.carried = self.safe_insert(target, carried, amount);
                    } else if count(&carried) <= self.slot_max_for(held) {
                        *self.carried = in_slot.clone();
                        self.put(target, carried);
                    }
                }
                Some(held) if same_item_same_components(existing, held) => {
                    let room = max_stack(held).saturating_sub(count(&carried));
                    if let Some(taken) = self.try_remove(target, count(&in_slot), room) {
                        *self.carried = with_count(held, count(&carried) + count(&Some(taken)));
                    }
                }
                Some(_) => {}
            },
        }
    }

    fn quick_move_stack(&mut self, slot: usize) -> Slot {
        let copy = self.item(slot)?;
        let mut stack = Some(copy.clone());
        let mut moved = false;
        for (start, end, reverse) in self.kind.quick_move_plan(slot, &copy, self.container) {
            if self.move_to(&mut stack, start, end, reverse) {
                moved = true;
                break;
            }
        }
        if !moved {
            return None;
        }
        self.put(slot, stack.clone());
        if count(&stack) == u32::from(copy.count.get()) {
            return None;
        }
        Some(copy)
    }

    fn move_to(&mut self, stack: &mut Slot, start: usize, end: usize, reverse: bool) -> bool {
        let order: Vec<usize> = if reverse {
            (start..end).rev().collect()
        } else {
            (start..end).collect()
        };
        let mut moved = false;
        if stack.as_ref().is_some_and(|s| max_stack(s) > 1) {
            for &target in &order {
                let Some(incoming) = stack.clone() else {
                    break;
                };
                let Some(existing) = self.item(target) else {
                    continue;
                };
                if !same_item_same_components(&incoming, &existing) {
                    continue;
                }
                let total = u32::from(existing.count.get()) + u32::from(incoming.count.get());
                let limit = self.slot_max_for(&existing);
                if total <= limit {
                    *stack = None;
                    self.put(target, with_count(&existing, total));
                    moved = true;
                } else if u32::from(existing.count.get()) < limit {
                    split(stack, limit - u32::from(existing.count.get()));
                    self.put(target, with_count(&existing, limit));
                    moved = true;
                }
            }
        }
        if let Some(incoming) = stack.clone() {
            for &target in &order {
                if self.item(target).is_none() && self.may_place(target, stack) {
                    let limit = self.slot_max_for(&incoming);
                    let part = split(stack, u32::from(incoming.count.get()).min(limit));
                    self.put(target, part);
                    moved = true;
                    break;
                }
            }
        }
        moved
    }

    fn swap(&mut self, slot: i32, button: i32) {
        let Some(target) = self.index(slot) else {
            return;
        };
        let Some(own) = usize::try_from(button).ok().and_then(menu_slot) else {
            return;
        };
        let held = self.player.get(own).cloned();
        let in_slot = self.item(target);
        match (&held, &in_slot) {
            (None, None) => {}
            (None, Some(_)) => {
                self.player.set(own, in_slot.clone());
                self.put(target, None);
            }
            (Some(stack), None) => {
                if self.may_place(target, &held) {
                    let limit = self.slot_max_for(stack);
                    if count(&held) > limit {
                        let mut rest = held.clone();
                        let part = split(&mut rest, limit);
                        self.player.set(own, rest);
                        self.put(target, part);
                    } else {
                        self.player.set(own, None);
                        self.put(target, held.clone());
                    }
                }
            }
            (Some(stack), Some(_)) => {
                if self.may_place(target, &held) {
                    let limit = self.slot_max_for(stack);
                    if count(&held) > limit {
                        let mut rest = held.clone();
                        let part = split(&mut rest, limit);
                        self.player.set(own, rest);
                        self.put(target, part);
                        let mut displaced = in_slot.clone();
                        if !self.player.add(&mut displaced, self.infinite) {
                            self.drop_stack(displaced, true, true);
                        }
                    } else {
                        self.player.set(own, in_slot.clone());
                        self.put(target, held.clone());
                    }
                }
            }
        }
    }

    fn clone_slot(&mut self, slot: i32) {
        if !self.infinite || self.carried.is_some() {
            return;
        }
        let Some(target) = self.index(slot) else {
            return;
        };
        if let Some(stack) = self.item(target) {
            *self.carried = with_count(&stack, max_stack(&stack));
        }
    }

    fn throw(&mut self, slot: i32, button: i32) {
        if self.carried.is_some() {
            return;
        }
        let Some(target) = self.index(slot) else {
            return;
        };
        let amount = if button == 0 {
            1
        } else {
            count(&self.item(target))
        };
        let mut dropped = self.try_remove(target, amount, u32::MAX);
        self.drop_stack(dropped.clone(), true, true);
        if button == 1 {
            while dropped.is_some() && same_item(&self.item(target), &dropped) {
                dropped = self.try_remove(target, amount, u32::MAX);
                if !self.drop_stack(dropped.clone(), true, true) {
                    break;
                }
            }
        }
    }

    fn pickup_all(&mut self, slot: i32, button: i32) {
        let Some(target) = self.index(slot) else {
            return;
        };
        if self.carried.is_none() || self.item(target).is_some() {
            return;
        }
        let total = self.kind.slot_count();
        let order: Vec<usize> = if button == 0 {
            (0..total).collect()
        } else {
            (0..total).rev().collect()
        };
        for pass in 0..2 {
            for &source in &order {
                let Some(carried) = self.carried.clone() else {
                    return;
                };
                if u32::from(carried.count.get()) >= max_stack(&carried) {
                    break;
                }
                let Some(existing) = self.item(source) else {
                    continue;
                };
                if !self.can_quick_replace(source, &carried) {
                    continue;
                }
                if pass == 0 && u32::from(existing.count.get()) == max_stack(&existing) {
                    continue;
                }
                let room = max_stack(&carried) - u32::from(carried.count.get());
                let taken = self.try_remove(source, u32::from(existing.count.get()), room);
                *self.carried =
                    with_count(&carried, u32::from(carried.count.get()) + count(&taken));
            }
        }
    }
}

impl ContainerMenu {
    pub fn new(kind: MenuKind) -> Self {
        Self {
            kind,
            carried: None,
            quick_craft: QuickCraft::default(),
            remote: vec![RemoteSlot::Known(None); kind.slot_count()],
            remote_carried: RemoteSlot::Known(None),
        }
    }

    pub fn kind(&self) -> MenuKind {
        self.kind
    }

    pub fn carried(&self) -> &Slot {
        &self.carried
    }

    pub fn click(
        &mut self,
        container: &mut Vec<Slot>,
        player: &mut Inventory,
        slot: i16,
        button: i8,
        input: ClickInput,
        clicker: Clicker,
    ) -> Vec<MenuEffect> {
        container.resize(self.kind.container_size(), None);
        let mut session = Session {
            kind: self.kind,
            container,
            player,
            carried: &mut self.carried,
            infinite: clicker.infinite_materials,
            effects: Vec::new(),
        };
        session.click(
            &mut self.quick_craft,
            i32::from(slot),
            i32::from(button),
            input,
        );
        session.effects
    }

    pub fn close(&mut self, player: &mut Inventory) -> Vec<MenuEffect> {
        let mut effects = Vec::new();
        if let Some(carried) = self.carried.take() {
            place_back_or_drop(player, carried, &mut effects);
        }
        effects
    }

    pub fn close_transient(
        &mut self,
        container: &mut [Slot],
        player: &mut Inventory,
    ) -> Vec<MenuEffect> {
        let mut effects = self.close(player);
        for (slot, entry) in container.iter_mut().enumerate() {
            if self.kind.clears_on_close(slot)
                && let Some(stack) = entry.take()
            {
                place_back_or_drop(player, stack, &mut effects);
            }
        }
        effects
    }

    pub fn receive_remote(&mut self, slot: i16, hash: HashedSlot) -> bool {
        match usize::try_from(slot)
            .ok()
            .and_then(|slot| self.remote.get_mut(slot))
        {
            Some(remote) => {
                *remote = RemoteSlot::Hashed(hash);
                true
            }
            None => false,
        }
    }

    pub fn receive_remote_carried(&mut self, hash: HashedSlot) {
        self.remote_carried = RemoteSlot::Hashed(hash);
    }

    fn view(&self, slot: usize, container: &[Slot], player: &Inventory) -> Slot {
        let size = self.kind.container_size();
        if slot < size {
            container.get(slot).cloned().flatten()
        } else if slot < size + MAIN_SLOTS {
            player.get(MAIN_START + slot - size).cloned()
        } else {
            player.get(HOTBAR_START + slot - size - MAIN_SLOTS).cloned()
        }
    }

    pub fn changes(&mut self, container: &[Slot], player: &Inventory) -> Vec<RemoteUpdate> {
        let mut updates = Vec::new();
        for slot in 0..self.remote.len() {
            let stack = self.view(slot, container, player);
            if !self.remote[slot].matches(&stack) {
                self.remote[slot] = RemoteSlot::Known(stack.clone());
                updates.push(RemoteUpdate::Slot(slot, stack));
            }
        }
        if !self.remote_carried.matches(&self.carried) {
            self.remote_carried = RemoteSlot::Known(self.carried.clone());
            updates.push(RemoteUpdate::Carried(self.carried.clone()));
        }
        updates
    }

    pub fn sync_all(&mut self, container: &[Slot], player: &Inventory) -> (Vec<Slot>, Slot) {
        let slots: Vec<Slot> = (0..self.remote.len())
            .map(|slot| self.view(slot, container, player))
            .collect();
        for (remote, stack) in self.remote.iter_mut().zip(&slots) {
            *remote = RemoteSlot::Known(stack.clone());
        }
        self.remote_carried = RemoteSlot::Known(self.carried.clone());
        (slots, self.carried.clone())
    }
}

#[cfg(test)]
mod tests {
    use std::num::NonZeroU8;

    use rspigot_nbt::Compound;

    use super::*;

    fn stack(name: &str, count: u8) -> ItemStack {
        let kind = ItemKind::from_name(name).expect("known item");
        ItemStack::new(kind.id(), NonZeroU8::new(count).expect("non zero"))
    }

    fn chest() -> (ContainerMenu, Vec<Slot>, Inventory) {
        (
            ContainerMenu::new(MenuKind::Generic { rows: 3 }),
            vec![None; 27],
            Inventory::default(),
        )
    }

    fn click(
        menu: &mut ContainerMenu,
        container: &mut Vec<Slot>,
        player: &mut Inventory,
        slot: i16,
        button: i8,
        input: ClickInput,
    ) -> Vec<MenuEffect> {
        menu.click(container, player, slot, button, input, Clicker::default())
    }

    #[test]
    fn the_menu_ids_match_the_registry() {
        let registry = |name: &str| {
            let index = rspigot_data::registries::static_index("minecraft:menu", name).expect(name);
            i32::try_from(index).expect("small id")
        };
        assert_eq!(
            MenuKind::Generic { rows: 1 }.protocol_id(),
            registry("minecraft:generic_9x1")
        );
        assert_eq!(
            MenuKind::Generic { rows: 3 }.protocol_id(),
            registry("minecraft:generic_9x3")
        );
        assert_eq!(
            MenuKind::Generic { rows: 6 }.protocol_id(),
            registry("minecraft:generic_9x6")
        );
        assert_eq!(
            MenuKind::Generic3x3.protocol_id(),
            registry("minecraft:generic_3x3")
        );
        assert_eq!(MenuKind::Hopper.protocol_id(), registry("minecraft:hopper"));
        assert_eq!(
            MenuKind::ShulkerBox.protocol_id(),
            registry("minecraft:shulker_box")
        );
        assert_eq!(
            MenuKind::Crafting.protocol_id(),
            registry("minecraft:crafting")
        );
        assert_eq!(
            MenuKind::Furnace.protocol_id(),
            registry("minecraft:furnace")
        );
        assert_eq!(
            MenuKind::BlastFurnace.protocol_id(),
            registry("minecraft:blast_furnace")
        );
        assert_eq!(MenuKind::Smoker.protocol_id(), registry("minecraft:smoker"));
    }

    fn workstation(kind: MenuKind) -> (ContainerMenu, Vec<Slot>, Inventory) {
        (
            ContainerMenu::new(kind),
            vec![None; kind.container_size()],
            Inventory::default(),
        )
    }

    #[test]
    fn the_crafting_menu_has_a_result_slot_that_takes_nothing() {
        let (mut menu, mut container, mut player) = workstation(MenuKind::Crafting);
        assert_eq!(MenuKind::Crafting.slot_count(), 46);
        player.set(MAIN_START, Some(stack("minecraft:stick", 4)));
        click(
            &mut menu,
            &mut container,
            &mut player,
            10,
            0,
            ClickInput::Pickup,
        );
        click(
            &mut menu,
            &mut container,
            &mut player,
            0,
            0,
            ClickInput::Pickup,
        );
        assert!(container[0].is_none());
        assert!(menu.carried().is_some());
        click(
            &mut menu,
            &mut container,
            &mut player,
            5,
            0,
            ClickInput::Pickup,
        );
        assert_eq!(container[5].as_ref().map(|s| s.count.get()), Some(4));
    }

    #[test]
    fn shift_click_in_the_crafting_menu_fills_the_grid_then_the_hotbar() {
        let (mut menu, mut container, mut player) = workstation(MenuKind::Crafting);
        player.set(MAIN_START, Some(stack("minecraft:stick", 2)));
        player.set(HOTBAR_START, Some(stack("minecraft:stone", 3)));
        click(
            &mut menu,
            &mut container,
            &mut player,
            10,
            0,
            ClickInput::QuickMove,
        );
        assert_eq!(container[1].as_ref().map(|s| s.count.get()), Some(2));
        click(
            &mut menu,
            &mut container,
            &mut player,
            37,
            0,
            ClickInput::QuickMove,
        );
        assert_eq!(container[2].as_ref().map(|s| s.count.get()), Some(3));
        click(
            &mut menu,
            &mut container,
            &mut player,
            1,
            0,
            ClickInput::QuickMove,
        );
        assert!(container[1].is_none());
        assert_eq!(player.get(MAIN_START).map(|s| s.count.get()), Some(2));
    }

    #[test]
    fn closing_the_crafting_menu_gives_the_grid_and_the_stack_in_hand_back() {
        let (mut menu, mut container, mut player) = workstation(MenuKind::Crafting);
        container[3] = Some(stack("minecraft:diamond", 2));
        container[0] = Some(stack("minecraft:stick", 1));
        player.set(MAIN_START, Some(stack("minecraft:stone", 5)));
        click(
            &mut menu,
            &mut container,
            &mut player,
            10,
            0,
            ClickInput::Pickup,
        );
        let effects = menu.close_transient(&mut container, &mut player);
        assert!(container[3].is_none());
        assert!(container[0].is_some(), "the result slot is not cleared");
        assert_eq!(effects.len(), 2);
        let held: u32 = (0..46)
            .filter_map(|slot| player.get(slot))
            .map(|s| u32::from(s.count.get()))
            .sum();
        assert_eq!(held, 7);
    }

    #[test]
    fn the_furnace_result_slot_refuses_items_and_shift_click_swaps_main_and_hotbar() {
        let (mut menu, mut container, mut player) = workstation(MenuKind::Furnace);
        assert_eq!(MenuKind::Furnace.slot_count(), 39);
        assert_eq!(MenuKind::Furnace.data_count(), 4);
        player.set(MAIN_START, Some(stack("minecraft:iron_ore", 3)));
        click(
            &mut menu,
            &mut container,
            &mut player,
            3,
            0,
            ClickInput::Pickup,
        );
        click(
            &mut menu,
            &mut container,
            &mut player,
            2,
            0,
            ClickInput::Pickup,
        );
        assert!(container[2].is_none());
        click(
            &mut menu,
            &mut container,
            &mut player,
            3,
            0,
            ClickInput::Pickup,
        );
        click(
            &mut menu,
            &mut container,
            &mut player,
            3,
            0,
            ClickInput::QuickMove,
        );
        assert_eq!(player.get(HOTBAR_START).map(|s| s.count.get()), Some(3));
        click(
            &mut menu,
            &mut container,
            &mut player,
            30,
            0,
            ClickInput::QuickMove,
        );
        assert_eq!(player.get(MAIN_START).map(|s| s.count.get()), Some(3));
    }

    #[test]
    fn the_furnace_slots_accept_items_by_hand_and_give_them_back_with_shift() {
        let (mut menu, mut container, mut player) = workstation(MenuKind::Smoker);
        player.set(MAIN_START, Some(stack("minecraft:beef", 2)));
        click(
            &mut menu,
            &mut container,
            &mut player,
            3,
            0,
            ClickInput::Pickup,
        );
        click(
            &mut menu,
            &mut container,
            &mut player,
            0,
            0,
            ClickInput::Pickup,
        );
        assert_eq!(container[0].as_ref().map(|s| s.count.get()), Some(2));
        click(
            &mut menu,
            &mut container,
            &mut player,
            0,
            0,
            ClickInput::QuickMove,
        );
        assert!(container[0].is_none());
        assert_eq!(player.get(MAIN_START).map(|s| s.count.get()), Some(2));
    }

    #[test]
    fn slots_are_the_container_then_the_main_inventory_then_the_hotbar() {
        let kind = MenuKind::Generic { rows: 3 };
        assert_eq!(kind.container_size(), 27);
        assert_eq!(kind.slot_count(), 27 + 27 + 9);
        let (mut menu, mut container, mut player) = chest();
        player.set(HOTBAR_START, Some(stack("minecraft:diamond", 2)));
        player.set(MAIN_START, Some(stack("minecraft:stick", 3)));
        let (slots, _) = menu.sync_all(&container, &player);
        assert_eq!(slots.len(), 63);
        assert_eq!(slots[27].as_ref().map(|s| s.count.get()), Some(3));
        assert_eq!(slots[54].as_ref().map(|s| s.count.get()), Some(2));
        container[0] = Some(stack("minecraft:stone", 5));
        let (slots, _) = menu.sync_all(&container, &player);
        assert_eq!(slots[0].as_ref().map(|s| s.count.get()), Some(5));
    }

    #[test]
    fn a_click_moves_a_stack_into_the_container_and_back() {
        let (mut menu, mut container, mut player) = chest();
        player.set(MAIN_START, Some(stack("minecraft:cobblestone", 10)));
        click(
            &mut menu,
            &mut container,
            &mut player,
            27,
            0,
            ClickInput::Pickup,
        );
        assert_eq!(menu.carried().as_ref().map(|s| s.count.get()), Some(10));
        click(
            &mut menu,
            &mut container,
            &mut player,
            4,
            0,
            ClickInput::Pickup,
        );
        assert_eq!(container[4].as_ref().map(|s| s.count.get()), Some(10));
        assert!(menu.carried().is_none());
        assert!(player.get(MAIN_START).is_none());
        click(
            &mut menu,
            &mut container,
            &mut player,
            4,
            1,
            ClickInput::Pickup,
        );
        assert_eq!(menu.carried().as_ref().map(|s| s.count.get()), Some(5));
        assert_eq!(container[4].as_ref().map(|s| s.count.get()), Some(5));
    }

    #[test]
    fn shift_click_fills_the_container_forward_and_the_inventory_backward() {
        let (mut menu, mut container, mut player) = chest();
        player.set(HOTBAR_START + 2, Some(stack("minecraft:apple", 4)));
        click(
            &mut menu,
            &mut container,
            &mut player,
            54 + 2,
            0,
            ClickInput::QuickMove,
        );
        assert_eq!(container[0].as_ref().map(|s| s.count.get()), Some(4));
        assert!(player.get(HOTBAR_START + 2).is_none());
        click(
            &mut menu,
            &mut container,
            &mut player,
            0,
            0,
            ClickInput::QuickMove,
        );
        assert!(container[0].is_none());
        assert_eq!(player.get(HOTBAR_START + 8).map(|s| s.count.get()), Some(4));
    }

    #[test]
    fn shift_click_tops_up_existing_stacks_first() {
        let (mut menu, mut container, mut player) = chest();
        container[3] = Some(stack("minecraft:cobblestone", 60));
        player.set(MAIN_START, Some(stack("minecraft:cobblestone", 10)));
        click(
            &mut menu,
            &mut container,
            &mut player,
            27,
            0,
            ClickInput::QuickMove,
        );
        assert_eq!(container[3].as_ref().map(|s| s.count.get()), Some(64));
        assert_eq!(container[0].as_ref().map(|s| s.count.get()), Some(6));
    }

    #[test]
    fn number_keys_swap_with_the_hotbar_and_f_with_the_offhand() {
        let (mut menu, mut container, mut player) = chest();
        container[1] = Some(stack("minecraft:gold_ingot", 3));
        player.set(HOTBAR_START + 4, Some(stack("minecraft:iron_ingot", 2)));
        click(
            &mut menu,
            &mut container,
            &mut player,
            1,
            4,
            ClickInput::Swap,
        );
        assert_eq!(
            container[1].as_ref().map(|s| s.item),
            Some(stack("minecraft:iron_ingot", 1).item)
        );
        assert_eq!(player.get(HOTBAR_START + 4).map(|s| s.count.get()), Some(3));
        click(
            &mut menu,
            &mut container,
            &mut player,
            1,
            40,
            ClickInput::Swap,
        );
        assert!(container[1].is_none());
        assert!(player.get(crate::item::OFFHAND_SLOT).is_some());
    }

    #[test]
    fn a_shulker_box_refuses_shulker_boxes_but_the_inventory_side_does_not() {
        let mut menu = ContainerMenu::new(MenuKind::ShulkerBox);
        let mut container = vec![None; 27];
        let mut player = Inventory::default();
        player.set(MAIN_START, Some(stack("minecraft:shulker_box", 1)));
        player.set(MAIN_START + 1, Some(stack("minecraft:stone", 1)));
        click(
            &mut menu,
            &mut container,
            &mut player,
            27,
            0,
            ClickInput::Pickup,
        );
        click(
            &mut menu,
            &mut container,
            &mut player,
            0,
            0,
            ClickInput::Pickup,
        );
        assert!(container[0].is_none());
        assert!(menu.carried().is_some());
        click(
            &mut menu,
            &mut container,
            &mut player,
            28,
            0,
            ClickInput::QuickMove,
        );
        assert_eq!(
            container[0].as_ref().map(|s| s.item),
            Some(stack("minecraft:stone", 1).item)
        );
        click(
            &mut menu,
            &mut container,
            &mut player,
            29,
            0,
            ClickInput::Pickup,
        );
        assert!(container.iter().flatten().all(|s| !is_shulker_box(s)));
    }

    #[test]
    fn clicking_outside_throws_the_carried_stack_or_one_of_it() {
        let (mut menu, mut container, mut player) = chest();
        player.set(MAIN_START, Some(stack("minecraft:stone", 8)));
        click(
            &mut menu,
            &mut container,
            &mut player,
            27,
            0,
            ClickInput::Pickup,
        );
        let effects = click(
            &mut menu,
            &mut container,
            &mut player,
            OUTSIDE,
            1,
            ClickInput::Pickup,
        );
        assert_eq!(
            effects,
            vec![MenuEffect::Drop {
                stack: stack("minecraft:stone", 1),
                thrower: true,
                predicted: true
            }]
        );
        let effects = click(
            &mut menu,
            &mut container,
            &mut player,
            OUTSIDE,
            0,
            ClickInput::Pickup,
        );
        assert_eq!(
            effects,
            vec![MenuEffect::Drop {
                stack: stack("minecraft:stone", 7),
                thrower: true,
                predicted: true
            }]
        );
    }

    #[test]
    fn dragging_splits_the_carried_stack_over_container_slots() {
        let (mut menu, mut container, mut player) = chest();
        player.set(MAIN_START, Some(stack("minecraft:stone", 9)));
        click(
            &mut menu,
            &mut container,
            &mut player,
            27,
            0,
            ClickInput::Pickup,
        );
        click(
            &mut menu,
            &mut container,
            &mut player,
            OUTSIDE,
            0,
            ClickInput::QuickCraft,
        );
        for slot in [0, 1, 2] {
            click(
                &mut menu,
                &mut container,
                &mut player,
                slot,
                1,
                ClickInput::QuickCraft,
            );
        }
        click(
            &mut menu,
            &mut container,
            &mut player,
            OUTSIDE,
            2,
            ClickInput::QuickCraft,
        );
        for (slot, stack) in container.iter().take(3).enumerate() {
            assert_eq!(stack.as_ref().map(|s| s.count.get()), Some(3), "{slot}");
        }
        assert!(menu.carried().is_none());
    }

    #[test]
    fn double_click_gathers_matching_stacks_from_both_sides() {
        let (mut menu, mut container, mut player) = chest();
        container[5] = Some(stack("minecraft:stone", 10));
        player.set(MAIN_START, Some(stack("minecraft:stone", 20)));
        player.set(MAIN_START + 1, Some(stack("minecraft:dirt", 1)));
        click(
            &mut menu,
            &mut container,
            &mut player,
            28,
            0,
            ClickInput::Pickup,
        );
        click(
            &mut menu,
            &mut container,
            &mut player,
            28,
            0,
            ClickInput::Pickup,
        );
        click(
            &mut menu,
            &mut container,
            &mut player,
            5,
            0,
            ClickInput::Pickup,
        );
        click(
            &mut menu,
            &mut container,
            &mut player,
            6,
            0,
            ClickInput::PickupAll,
        );
        assert_eq!(menu.carried().as_ref().map(|s| s.count.get()), Some(30));
    }

    #[test]
    fn closing_returns_the_carried_stack_to_the_inventory() {
        let (mut menu, mut container, mut player) = chest();
        container[0] = Some(stack("minecraft:diamond", 4));
        click(
            &mut menu,
            &mut container,
            &mut player,
            0,
            0,
            ClickInput::Pickup,
        );
        let effects = menu.close(&mut player);
        assert!(menu.carried().is_none());
        assert!(matches!(
            effects.as_slice(),
            [MenuEffect::PlacedBack { .. }]
        ));
        let held: u32 = (0..46)
            .filter_map(|slot| player.get(slot))
            .map(|s| u32::from(s.count.get()))
            .sum();
        assert_eq!(held, 4);
    }

    #[test]
    fn remote_slots_only_report_what_the_client_does_not_already_know() {
        let (mut menu, mut container, player) = chest();
        menu.sync_all(&container, &player);
        assert!(menu.changes(&container, &player).is_empty());
        container[2] = Some(stack("minecraft:stone", 1));
        let updates = menu.changes(&container, &player);
        assert_eq!(updates.len(), 1);
        assert!(menu.changes(&container, &player).is_empty());
        menu.receive_remote(2, HashedSlot::of(&None));
        assert_eq!(menu.changes(&container, &player).len(), 1);
    }

    const STATIONS: [(MenuKind, usize); 6] = [
        (MenuKind::Anvil, 2),
        (MenuKind::Smithing, 3),
        (MenuKind::Grindstone, 2),
        (MenuKind::Stonecutter, 1),
        (MenuKind::CartographyTable, 2),
        (MenuKind::Loom, 3),
    ];

    fn with_component(
        mut stack: ItemStack,
        key: &str,
        value: impl Into<rspigot_nbt::Tag>,
    ) -> ItemStack {
        stack.components.insert(key, value);
        stack
    }

    fn enchanted(name: &str, key: &str) -> ItemStack {
        with_component(
            stack(name, 1),
            key,
            Compound::new().with("minecraft:sharpness", 1),
        )
    }

    fn place_in(
        kind: MenuKind,
        container: &mut Vec<Slot>,
        player: &mut Inventory,
        menu: &mut ContainerMenu,
        slot: usize,
        item: ItemStack,
    ) -> bool {
        player.set(MAIN_START, Some(item));
        click(
            menu,
            container,
            player,
            kind.container_size() as i16,
            0,
            ClickInput::Pickup,
        );
        click(menu, container, player, slot as i16, 0, ClickInput::Pickup);
        let accepted = container[slot].is_some();
        if !accepted {
            click(
                menu,
                container,
                player,
                kind.container_size() as i16,
                0,
                ClickInput::Pickup,
            );
        }
        accepted
    }

    fn accepts_at(kind: MenuKind, slot: usize, item: ItemStack) -> bool {
        let (mut menu, mut container, mut player) = workstation(kind);
        place_in(kind, &mut container, &mut player, &mut menu, slot, item)
    }

    #[test]
    fn the_station_menu_ids_match_the_registry() {
        let registry = |name: &str| {
            let index = rspigot_data::registries::static_index("minecraft:menu", name).expect(name);
            i32::try_from(index).expect("small id")
        };
        assert_eq!(MenuKind::Anvil.protocol_id(), registry("minecraft:anvil"));
        assert_eq!(
            MenuKind::Smithing.protocol_id(),
            registry("minecraft:smithing")
        );
        assert_eq!(
            MenuKind::Grindstone.protocol_id(),
            registry("minecraft:grindstone")
        );
        assert_eq!(
            MenuKind::Stonecutter.protocol_id(),
            registry("minecraft:stonecutter")
        );
        assert_eq!(
            MenuKind::CartographyTable.protocol_id(),
            registry("minecraft:cartography_table")
        );
        assert_eq!(MenuKind::Loom.protocol_id(), registry("minecraft:loom"));
    }

    #[test]
    fn each_station_has_its_vanilla_slots_and_data_slots() {
        let expected = [
            (MenuKind::Anvil, 3, 1),
            (MenuKind::Smithing, 4, 1),
            (MenuKind::Grindstone, 3, 0),
            (MenuKind::Stonecutter, 2, 1),
            (MenuKind::CartographyTable, 3, 0),
            (MenuKind::Loom, 4, 1),
        ];
        for (kind, container, data) in expected {
            assert_eq!(kind.container_size(), container, "{kind:?}");
            assert_eq!(kind.slot_count(), container + 36, "{kind:?}");
            assert_eq!(kind.data_count(), data, "{kind:?}");
        }
    }

    #[test]
    fn the_result_slot_of_a_station_takes_nothing() {
        for (kind, result) in STATIONS {
            let accepted = accepts_at(kind, result, stack("minecraft:stone", 1));
            assert!(!accepted, "{kind:?}");
        }
    }

    #[test]
    fn closing_a_station_gives_the_inputs_back_and_keeps_the_result() {
        for (kind, result) in STATIONS {
            let (mut menu, mut container, mut player) = workstation(kind);
            for entry in container.iter_mut().take(result + 1) {
                *entry = Some(stack("minecraft:stone", 1));
            }
            menu.close_transient(&mut container, &mut player);
            assert!(container[..result].iter().all(Option::is_none), "{kind:?}");
            assert!(container[result].is_some(), "{kind:?}");
            let held: u32 = (0..46)
                .filter_map(|slot| player.get(slot))
                .map(|s| u32::from(s.count.get()))
                .sum();
            assert_eq!(held, result as u32, "{kind:?}");
        }
    }

    fn shift_click(kind: MenuKind, container: &mut Vec<Slot>, player: &mut Inventory, slot: usize) {
        let mut menu = ContainerMenu::new(kind);
        click(
            &mut menu,
            container,
            player,
            slot as i16,
            0,
            ClickInput::QuickMove,
        );
    }

    #[test]
    fn shift_clicking_a_station_result_fills_the_inventory_from_the_end() {
        for (kind, result) in STATIONS {
            let (mut container, mut player) =
                (vec![None; kind.container_size()], Inventory::default());
            container[result] = Some(stack("minecraft:stone", 1));
            shift_click(kind, &mut container, &mut player, result);
            assert!(container[result].is_none(), "{kind:?}");
            assert_eq!(
                player.get(HOTBAR_START + 8).map(|s| s.count.get()),
                Some(1),
                "{kind:?}"
            );
        }
    }

    #[test]
    fn the_anvil_inputs_take_any_item_and_shift_click_fills_them_before_anything_else() {
        let kind = MenuKind::Anvil;
        let (mut menu, mut container, mut player) = workstation(kind);
        assert!(place_in(
            kind,
            &mut container,
            &mut player,
            &mut menu,
            0,
            stack("minecraft:stone", 1)
        ));
        assert!(place_in(
            kind,
            &mut container,
            &mut player,
            &mut menu,
            1,
            stack("minecraft:stick", 1)
        ));

        let (mut container, mut player) = (vec![None; 3], Inventory::default());
        player.set(MAIN_START, Some(stack("minecraft:stone", 2)));
        player.set(HOTBAR_START, Some(stack("minecraft:stick", 3)));
        player.set(HOTBAR_START + 1, Some(stack("minecraft:dirt", 4)));
        shift_click(kind, &mut container, &mut player, 3);
        shift_click(kind, &mut container, &mut player, 30);
        assert_eq!(container[0].as_ref().map(|s| s.count.get()), Some(2));
        assert_eq!(container[1].as_ref().map(|s| s.count.get()), Some(3));
        shift_click(kind, &mut container, &mut player, 31);
        assert_eq!(player.get(HOTBAR_START + 1).map(|s| s.count.get()), Some(4));
        assert!(player.get(MAIN_START).is_none());
        shift_click(kind, &mut container, &mut player, 0);
        assert!(container[0].is_none());
        assert_eq!(player.get(MAIN_START).map(|s| s.count.get()), Some(2));
    }

    #[test]
    fn the_smithing_inputs_take_any_item_and_shift_click_only_swaps_main_and_hotbar() {
        let kind = MenuKind::Smithing;
        let (mut menu, mut container, mut player) = workstation(kind);
        for slot in 0..3 {
            assert!(
                place_in(
                    kind,
                    &mut container,
                    &mut player,
                    &mut menu,
                    slot,
                    stack("minecraft:stone", 1)
                ),
                "{slot}"
            );
        }
        let (mut container, mut player) = (vec![None; 4], Inventory::default());
        player.set(MAIN_START, Some(stack("minecraft:netherite_ingot", 1)));
        shift_click(kind, &mut container, &mut player, 4);
        assert!(container.iter().all(Option::is_none));
        assert!(player.get(MAIN_START).is_none());
        assert_eq!(player.get(HOTBAR_START).map(|s| s.count.get()), Some(1));
        shift_click(kind, &mut container, &mut player, 31);
        assert_eq!(player.get(MAIN_START).map(|s| s.count.get()), Some(1));
    }

    #[test]
    fn the_grindstone_takes_only_damageable_or_enchanted_items() {
        let kind = MenuKind::Grindstone;
        for slot in [0, 1] {
            assert!(
                accepts_at(kind, slot, stack("minecraft:iron_pickaxe", 1)),
                "pickaxe {slot}"
            );
            assert!(
                accepts_at(
                    kind,
                    slot,
                    enchanted("minecraft:stone", "minecraft:enchantments")
                ),
                "enchanted {slot}"
            );
            assert!(
                accepts_at(
                    kind,
                    slot,
                    enchanted("minecraft:book", "minecraft:stored_enchantments")
                ),
                "stored {slot}"
            );
            assert!(
                !accepts_at(kind, slot, stack("minecraft:stick", 1)),
                "stick {slot}"
            );
            assert!(
                !accepts_at(
                    kind,
                    slot,
                    with_component(
                        stack("minecraft:stone", 1),
                        "minecraft:enchantments",
                        Compound::new()
                    )
                ),
                "empty {slot}"
            );
            assert!(
                !accepts_at(
                    kind,
                    slot,
                    with_component(
                        stack("minecraft:iron_pickaxe", 1),
                        "minecraft:unbreakable",
                        Compound::new()
                    )
                ),
                "unbreakable {slot}"
            );
        }
    }

    #[test]
    fn grindstone_shift_click_fills_the_inputs_then_swaps_main_and_hotbar() {
        let kind = MenuKind::Grindstone;
        let (mut container, mut player) = (vec![None; 3], Inventory::default());
        player.set(MAIN_START, Some(stack("minecraft:stick", 1)));
        shift_click(kind, &mut container, &mut player, 3);
        assert!(container.iter().all(Option::is_none));
        assert!(player.get(MAIN_START).is_some());

        player.set(MAIN_START + 1, Some(stack("minecraft:iron_pickaxe", 1)));
        shift_click(kind, &mut container, &mut player, 4);
        assert!(container[0].is_some());
        player.set(MAIN_START + 2, Some(stack("minecraft:iron_axe", 1)));
        shift_click(kind, &mut container, &mut player, 5);
        assert!(container[1].is_some());

        shift_click(kind, &mut container, &mut player, 3);
        assert!(player.get(MAIN_START).is_none());
        assert!(player.get(HOTBAR_START).is_some());
    }

    #[test]
    fn the_stonecutter_input_takes_any_item_and_shift_click_swaps_main_and_hotbar() {
        let kind = MenuKind::Stonecutter;
        assert!(accepts_at(kind, 0, stack("minecraft:stone", 1)));
        assert!(accepts_at(kind, 0, stack("minecraft:stick", 1)));
        let (mut container, mut player) = (vec![None; 2], Inventory::default());
        player.set(MAIN_START, Some(stack("minecraft:stone", 5)));
        shift_click(kind, &mut container, &mut player, 2);
        assert!(container.iter().all(Option::is_none));
        assert_eq!(player.get(HOTBAR_START).map(|s| s.count.get()), Some(5));
        shift_click(kind, &mut container, &mut player, 29);
        assert_eq!(player.get(MAIN_START).map(|s| s.count.get()), Some(5));
    }

    #[test]
    fn the_cartography_table_takes_a_filled_map_then_paper_a_map_or_a_glass_pane() {
        let kind = MenuKind::CartographyTable;
        let filled = with_component(stack("minecraft:filled_map", 1), "minecraft:map_id", 3);
        assert!(accepts_at(kind, 0, filled.clone()));
        assert!(!accepts_at(kind, 0, stack("minecraft:filled_map", 1)));
        assert!(!accepts_at(kind, 0, stack("minecraft:paper", 1)));
        assert!(!accepts_at(kind, 1, filled));
        for name in ["minecraft:paper", "minecraft:map", "minecraft:glass_pane"] {
            assert!(accepts_at(kind, 1, stack(name, 1)), "{name}");
        }
        assert!(!accepts_at(kind, 1, stack("minecraft:stone", 1)));
    }

    #[test]
    fn cartography_shift_click_routes_maps_and_materials_to_their_slot() {
        let kind = MenuKind::CartographyTable;
        let (mut container, mut player) = (vec![None; 3], Inventory::default());
        player.set(
            MAIN_START,
            Some(with_component(
                stack("minecraft:filled_map", 1),
                "minecraft:map_id",
                3,
            )),
        );
        player.set(MAIN_START + 1, Some(stack("minecraft:paper", 2)));
        player.set(MAIN_START + 2, Some(stack("minecraft:stone", 2)));
        shift_click(kind, &mut container, &mut player, 3);
        shift_click(kind, &mut container, &mut player, 4);
        shift_click(kind, &mut container, &mut player, 5);
        assert!(container[0].is_some());
        assert_eq!(container[1].as_ref().map(|s| s.count.get()), Some(2));
        assert!(player.get(MAIN_START + 2).is_none());
        assert_eq!(player.get(HOTBAR_START).map(|s| s.count.get()), Some(2));
    }

    #[test]
    fn the_loom_takes_a_banner_a_dye_and_a_pattern_in_their_own_slot() {
        let kind = MenuKind::Loom;
        assert!(accepts_at(kind, 0, stack("minecraft:white_banner", 1)));
        assert!(accepts_at(kind, 0, stack("minecraft:red_banner", 1)));
        assert!(!accepts_at(kind, 0, stack("minecraft:red_dye", 1)));
        assert!(accepts_at(kind, 1, stack("minecraft:red_dye", 1)));
        assert!(accepts_at(kind, 1, stack("minecraft:black_dye", 1)));
        assert!(!accepts_at(kind, 1, stack("minecraft:white_banner", 1)));
        assert!(accepts_at(
            kind,
            2,
            stack("minecraft:creeper_banner_pattern", 1)
        ));
        assert!(!accepts_at(kind, 2, stack("minecraft:red_dye", 1)));
        assert!(!accepts_at(kind, 2, stack("minecraft:stone", 1)));
    }

    #[test]
    fn loom_shift_click_routes_each_item_to_its_slot_and_swaps_the_rest() {
        let kind = MenuKind::Loom;
        let (mut container, mut player) = (vec![None; 4], Inventory::default());
        player.set(MAIN_START, Some(stack("minecraft:white_banner", 1)));
        player.set(MAIN_START + 1, Some(stack("minecraft:red_dye", 3)));
        player.set(
            MAIN_START + 2,
            Some(stack("minecraft:creeper_banner_pattern", 1)),
        );
        player.set(MAIN_START + 3, Some(stack("minecraft:stone", 1)));
        for slot in 4..8 {
            shift_click(kind, &mut container, &mut player, slot);
        }
        assert!(container[0].is_some());
        assert_eq!(container[1].as_ref().map(|s| s.count.get()), Some(3));
        assert!(container[2].is_some());
        assert!(container[3].is_none());
        assert_eq!(player.get(HOTBAR_START).map(|s| s.count.get()), Some(1));
        assert!(player.get(MAIN_START + 3).is_none());
    }
}
