use rspigot_plugin_protocol::v1::{self, answer, from_plugin, inventory_order, question};
use uuid::Uuid;

use crate::link::Link;
use crate::{Error, ItemStack, Material};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ArmorSlot {
    Helmet,
    Chestplate,
    Leggings,
    Boots,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum InventorySlot {
    Hotbar(u8),
    Main(u8),
    Armor(ArmorSlot),
    Offhand,
}

impl InventorySlot {
    pub const fn index(self) -> Option<i32> {
        match self {
            Self::Hotbar(slot) if slot < 9 => Some(slot as i32),
            Self::Main(slot) if slot < 27 => Some(9 + slot as i32),
            Self::Hotbar(_) | Self::Main(_) => None,
            Self::Armor(ArmorSlot::Boots) => Some(36),
            Self::Armor(ArmorSlot::Leggings) => Some(37),
            Self::Armor(ArmorSlot::Chestplate) => Some(38),
            Self::Armor(ArmorSlot::Helmet) => Some(39),
            Self::Offhand => Some(40),
        }
    }

    pub const fn from_index(index: i32) -> Option<Self> {
        match index {
            0..=8 => Some(Self::Hotbar(index as u8)),
            9..=35 => Some(Self::Main((index - 9) as u8)),
            36 => Some(Self::Armor(ArmorSlot::Boots)),
            37 => Some(Self::Armor(ArmorSlot::Leggings)),
            38 => Some(Self::Armor(ArmorSlot::Chestplate)),
            39 => Some(Self::Armor(ArmorSlot::Helmet)),
            40 => Some(Self::Offhand),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InventoryState {
    pub slots: Vec<(InventorySlot, Option<ItemStack>)>,
    pub held_slot: u8,
}

impl InventoryState {
    fn from_wire(state: v1::InventoryState) -> Result<Self, Error> {
        let slots = state
            .slots
            .into_iter()
            .map(|slot| {
                let index = InventorySlot::from_index(slot.slot)
                    .ok_or_else(|| Error::Malformed(format!("an unknown slot {}", slot.slot)))?;
                Ok((index, slot.item.map(ItemStack::from_wire).transpose()?))
            })
            .collect::<Result<_, Error>>()?;
        let held_slot = u8::try_from(state.held_slot)
            .ok()
            .filter(|slot| *slot < 9)
            .ok_or_else(|| Error::Malformed(format!("an unknown held slot {}", state.held_slot)))?;
        Ok(Self { slots, held_slot })
    }
}

#[derive(Debug, Clone)]
pub struct Inventory {
    player: Uuid,
    link: Link,
}

impl Inventory {
    pub(crate) const fn new(player: Uuid, link: Link) -> Self {
        Self { player, link }
    }

    pub fn add(&self, item: ItemStack) {
        self.order(inventory_order::Order::Add(item.into_wire()));
    }

    pub fn set(&self, slot: InventorySlot, item: Option<ItemStack>) {
        self.order(inventory_order::Order::Set(v1::SetSlot {
            slot: slot.index().unwrap_or(-1),
            item: item.map(ItemStack::into_wire),
        }));
    }

    pub fn remove(&self, material: Material, count: i32) {
        self.order(inventory_order::Order::Remove(v1::RemoveItems {
            key: material.key().to_owned(),
            count,
        }));
    }

    pub fn clear(&self) {
        self.order(inventory_order::Order::Clear(v1::Empty {}));
    }

    pub fn set_held_slot(&self, slot: u8) {
        self.order(inventory_order::Order::SetHeldSlot(i32::from(slot)));
    }

    pub async fn resolve(&self) -> Result<InventoryState, Error> {
        let state = self
            .link
            .ask(
                question::Question::Inventory(self.player.to_string()),
                "an inventory",
                |body| match body {
                    answer::Body::Inventory(state) => Some(state),
                    _ => None,
                },
            )
            .await?;
        InventoryState::from_wire(state)
    }

    fn order(&self, order: inventory_order::Order) {
        self.link
            .order(from_plugin::Body::Inventory(v1::InventoryOrder {
                player: self.player.to_string(),
                order: Some(order),
            }));
    }
}

#[cfg(test)]
mod tests {
    use super::InventorySlot;

    #[test]
    fn slots_round_trip_through_their_index() {
        for index in 0..=40 {
            assert_eq!(
                InventorySlot::from_index(index).unwrap().index(),
                Some(index)
            );
        }
        assert_eq!(InventorySlot::from_index(41), None);
    }
}
