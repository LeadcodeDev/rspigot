use std::sync::atomic::{AtomicU64, Ordering};

use rspigot_plugin_protocol::v1::{
    self, answer, clicked_slot, custom_inventory_order, from_plugin, question, server_order,
};

use crate::link::Link;
use crate::{Error, InventorySlot, ItemStack, Player};

static NEXT_INVENTORY: AtomicU64 = AtomicU64::new(1);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ClickedSlot {
    Menu(u16),
    Player(InventorySlot),
    Outside,
}

impl ClickedSlot {
    pub(crate) fn from_wire(slot: Option<v1::ClickedSlot>) -> Result<Self, Error> {
        let slot = slot
            .and_then(|slot| slot.slot)
            .ok_or_else(|| Error::Malformed("a click without `slot`".to_owned()))?;
        match slot {
            clicked_slot::Slot::Menu(index) => u16::try_from(index)
                .map(Self::Menu)
                .map_err(|_| Error::Malformed(format!("an unknown menu slot {index}"))),
            clicked_slot::Slot::Player(index) => InventorySlot::from_index(index)
                .map(Self::Player)
                .ok_or_else(|| Error::Malformed(format!("an unknown slot {index}"))),
            clicked_slot::Slot::Outside(_) => Ok(Self::Outside),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ClickKind {
    Left,
    Right,
    ShiftLeft,
    ShiftRight,
    NumberKey(u8),
    SwapOffhand,
    Drop,
    ControlDrop,
    DoubleClick,
    Middle,
}

impl ClickKind {
    pub(crate) fn from_wire(kind: i32, hotbar_key: i32) -> Result<Self, Error> {
        match v1::ClickKind::try_from(kind) {
            Ok(v1::ClickKind::Left) => Ok(Self::Left),
            Ok(v1::ClickKind::Right) => Ok(Self::Right),
            Ok(v1::ClickKind::ShiftLeft) => Ok(Self::ShiftLeft),
            Ok(v1::ClickKind::ShiftRight) => Ok(Self::ShiftRight),
            Ok(v1::ClickKind::NumberKey) => u8::try_from(hotbar_key)
                .ok()
                .filter(|key| *key < 9)
                .map(Self::NumberKey)
                .ok_or_else(|| Error::Malformed(format!("an unknown hotbar key {hotbar_key}"))),
            Ok(v1::ClickKind::SwapOffhand) => Ok(Self::SwapOffhand),
            Ok(v1::ClickKind::Drop) => Ok(Self::Drop),
            Ok(v1::ClickKind::ControlDrop) => Ok(Self::ControlDrop),
            Ok(v1::ClickKind::DoubleClick) => Ok(Self::DoubleClick),
            Ok(v1::ClickKind::Middle) => Ok(Self::Middle),
            Ok(v1::ClickKind::Unspecified) | Err(_) => {
                Err(Error::Malformed(format!("an unknown click kind {kind}")))
            }
        }
    }
}

#[derive(Debug, Clone)]
pub struct CustomInventoryState {
    pub items: Vec<Option<ItemStack>>,
    pub viewers: Vec<Player>,
}

impl CustomInventoryState {
    fn from_wire(state: v1::CustomInventoryState, link: &Link) -> Result<Self, Error> {
        let items = (0..)
            .zip(state.slots)
            .map(|(index, slot)| {
                if slot.slot != index {
                    return Err(Error::Malformed(format!(
                        "slot {} where slot {index} was expected",
                        slot.slot
                    )));
                }
                slot.item.map(ItemStack::from_wire).transpose()
            })
            .collect::<Result<_, Error>>()?;
        let viewers = state
            .viewers
            .into_iter()
            .map(|viewer| Player::from_wire(Some(viewer), link))
            .collect::<Result<_, Error>>()?;
        Ok(Self { items, viewers })
    }
}

#[derive(Debug, Clone)]
pub struct CustomInventory {
    id: u64,
    generation: u64,
    link: Link,
}

impl CustomInventory {
    pub(crate) fn create(link: &Link, rows: u8, title: String, locked: bool) -> Self {
        let inventory = Self::new(NEXT_INVENTORY.fetch_add(1, Ordering::Relaxed), link);
        link.order_from(
            inventory.generation,
            from_plugin::Body::Server(v1::ServerOrder {
                order: Some(server_order::Order::CreateInventory(v1::CreateInventory {
                    id: inventory.id,
                    rows: i32::from(rows),
                    title,
                    locked,
                })),
            }),
        );
        inventory
    }

    pub(crate) fn new(id: u64, link: &Link) -> Self {
        Self {
            id,
            generation: link.generation(),
            link: link.clone(),
        }
    }

    pub const fn id(&self) -> u64 {
        self.id
    }

    pub(crate) const fn generation(&self) -> u64 {
        self.generation
    }

    pub fn set(&self, slot: u16, item: Option<ItemStack>) {
        self.order(custom_inventory_order::Order::Set(v1::SetSlot {
            slot: i32::from(slot),
            item: item.map(ItemStack::into_wire),
        }));
    }

    pub fn clear(&self) {
        self.order(custom_inventory_order::Order::Clear(v1::Empty {}));
    }

    pub fn close(&self) {
        self.order(custom_inventory_order::Order::Close(v1::Empty {}));
    }

    pub fn discard(&self) {
        self.order(custom_inventory_order::Order::Discard(v1::Empty {}));
    }

    pub async fn resolve(&self) -> Result<CustomInventoryState, Error> {
        if self.link.generation() != self.generation {
            return Err(Error::NotFound);
        }
        let state = self
            .link
            .ask(
                question::Question::CustomInventory(self.id),
                "a custom inventory",
                |body| match body {
                    answer::Body::CustomInventory(state) => Some(state),
                    _ => None,
                },
            )
            .await?;
        CustomInventoryState::from_wire(state, &self.link)
    }

    fn order(&self, order: custom_inventory_order::Order) {
        self.link.order_from(
            self.generation,
            from_plugin::Body::CustomInventory(v1::CustomInventoryOrder {
                inventory: self.id,
                order: Some(order),
            }),
        );
    }
}
