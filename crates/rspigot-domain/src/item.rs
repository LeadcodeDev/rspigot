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
}
