use std::num::NonZeroU8;

use rspigot_nbt::Compound;

use crate::ids::ItemId;

#[derive(Debug, Clone, PartialEq)]
pub struct ItemStack {
    pub item: ItemId,
    pub count: NonZeroU8,
    pub components: Compound,
}

impl ItemStack {
    pub fn new(item: ItemId, count: NonZeroU8) -> Self {
        Self {
            item,
            count,
            components: Compound::new(),
        }
    }
}

pub type Slot = Option<ItemStack>;

pub const HOTBAR_START: usize = 36;
pub const PLAYER_INVENTORY_SIZE: usize = 46;

#[derive(Debug, Clone, PartialEq)]
pub struct Inventory {
    slots: Vec<Slot>,
    pub selected_hotbar: u8,
}

impl Default for Inventory {
    fn default() -> Self {
        Self {
            slots: vec![None; PLAYER_INVENTORY_SIZE],
            selected_hotbar: 0,
        }
    }
}

impl Inventory {
    pub fn get(&self, slot: usize) -> Option<&ItemStack> {
        self.slots.get(slot)?.as_ref()
    }

    pub fn set(&mut self, slot: usize, stack: Slot) -> bool {
        match self.slots.get_mut(slot) {
            Some(entry) => {
                *entry = stack;
                true
            }
            None => false,
        }
    }

    pub fn held(&self) -> Option<&ItemStack> {
        self.get(HOTBAR_START + usize::from(self.selected_hotbar))
    }

    pub fn slots(&self) -> &[Slot] {
        &self.slots
    }

    pub fn insert(&mut self, item: ItemId, count: u32, max_stack: u8) -> InsertOutcome {
        let max = u32::from(max_stack.max(1));
        let mut remaining = count;
        let mut changed = Vec::new();
        let order: Vec<usize> = (HOTBAR_START..HOTBAR_START + 9)
            .chain(9..HOTBAR_START)
            .collect();
        for &slot in &order {
            if remaining == 0 {
                break;
            }
            let Some(existing) = self.slots[slot].as_mut() else {
                continue;
            };
            if existing.item != item || !existing.components.is_empty() {
                continue;
            }
            let added = max
                .saturating_sub(u32::from(existing.count.get()))
                .min(remaining);
            if added > 0 {
                existing.count = NonZeroU8::new((u32::from(existing.count.get()) + added) as u8)
                    .unwrap_or(existing.count);
                remaining -= added;
                changed.push(slot);
            }
        }
        for &slot in &order {
            if remaining == 0 {
                break;
            }
            if self.slots[slot].is_none() {
                let added = max.min(remaining);
                if let Some(count) = NonZeroU8::new(added as u8) {
                    self.slots[slot] = Some(ItemStack::new(item, count));
                    remaining -= added;
                    changed.push(slot);
                }
            }
        }
        InsertOutcome { remaining, changed }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InsertOutcome {
    pub remaining: u32,
    pub changed: Vec<usize>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn insert_merges_hotbar_first_then_fills_empty_slots() {
        let mut inventory = Inventory::default();
        inventory.set(
            HOTBAR_START + 2,
            Some(ItemStack::new(ItemId(5), NonZeroU8::new(60).unwrap())),
        );
        let outcome = inventory.insert(ItemId(5), 10, 64);
        assert_eq!(outcome.remaining, 0);
        assert_eq!(outcome.changed, vec![HOTBAR_START + 2, HOTBAR_START]);
        assert_eq!(
            inventory.get(HOTBAR_START + 2).map(|s| s.count.get()),
            Some(64)
        );
        assert_eq!(inventory.get(HOTBAR_START).map(|s| s.count.get()), Some(6));
    }

    #[test]
    fn insert_reports_leftover_when_full() {
        let mut inventory = Inventory::default();
        for slot in 9..HOTBAR_START + 9 {
            inventory.set(
                slot,
                Some(ItemStack::new(ItemId(1), NonZeroU8::new(64).unwrap())),
            );
        }
        assert_eq!(inventory.insert(ItemId(2), 3, 64).remaining, 3);
        assert_eq!(inventory.insert(ItemId(1), 3, 64).remaining, 3);
    }
}
