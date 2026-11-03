use std::collections::BTreeMap;
use std::ops::RangeInclusive;

use rspigot_domain::container_menu::{ContainerMenu, MenuKind};
use rspigot_domain::item::{Inventory, OFFHAND_SLOT, Slot, menu_slot};
use rspigot_domain::menu::{ClickInput, Clicker, MenuEffect, OUTSIDE};
use rspigot_domain::player::{GameMode, PlayerId};
use rspigot_domain::text::Component;

use crate::containers::{next_window, sync_menu_view};
use crate::events::{ClientEvent, ContainerClick};
use crate::plugins::items::{spec_of, stack_of};
use crate::plugins::resolve::text;
use crate::plugins::{
    ClickKind, ClickedSlot, CustomInventoryOrder, CustomInventoryState, EventPayload,
    PlayerSnapshot,
};
use crate::ports::ClientLink;
use crate::server::Server;

pub const MAX_PLUGIN_INVENTORIES: usize = 1024;
const ROWS: RangeInclusive<u8> = 1..=6;
const ROW_SLOTS: usize = 9;
const MAIN_SLOTS: usize = 27;
const FIRST_MAIN: i32 = 9;
const NO_SLOT: i16 = -1;
const OFFHAND_BUTTON: i8 = 40;
const OFFHAND: i32 = 40;

type Key = (u64, u64);

struct PluginInventory {
    rows: u8,
    title: Component,
    locked: bool,
    items: Vec<Slot>,
}

#[derive(Default)]
pub(crate) struct PluginInventories {
    held: BTreeMap<Key, PluginInventory>,
}

impl PluginInventories {
    fn held_by(&self, connection: u64) -> usize {
        self.held
            .range((connection, 0)..=(connection, u64::MAX))
            .count()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct PluginMenu {
    pub window: i32,
    pub connection: u64,
    pub inventory: u64,
    pub menu: ContainerMenu,
    pub state_id: i32,
}

impl PluginMenu {
    const fn key(&self) -> Key {
        (self.connection, self.inventory)
    }
}

impl<L: ClientLink> Server<L> {
    pub(super) fn create_plugin_inventory(
        &mut self,
        connection: u64,
        id: u64,
        rows: i32,
        title: &str,
        locked: bool,
    ) -> Result<(), String> {
        let rows = u8::try_from(rows)
            .ok()
            .filter(|rows| ROWS.contains(rows))
            .ok_or_else(|| format!("rows {rows} is outside {}..={}", ROWS.start(), ROWS.end()))?;
        let inventories = &mut self.plugin_inventories;
        if inventories.held.contains_key(&(connection, id)) {
            return Err(format!("inventory {id} already exists"));
        }
        if inventories.held_by(connection) >= MAX_PLUGIN_INVENTORIES {
            return Err(format!(
                "the plugin already holds {MAX_PLUGIN_INVENTORIES} inventories"
            ));
        }
        inventories.held.insert(
            (connection, id),
            PluginInventory {
                rows,
                title: text(title),
                locked,
                items: vec![None; usize::from(rows) * ROW_SLOTS],
            },
        );
        Ok(())
    }

    pub(super) fn open_plugin_menu(&mut self, connection: u64, id: PlayerId, inventory: u64) {
        let key = (connection, inventory);
        if !self.plugin_inventories.held.contains_key(&key) {
            return;
        }
        self.close_open_menu(id, true);
        let (Some(held), Some(player)) = (
            self.plugin_inventories.held.get(&key),
            self.players.get_mut(&id),
        ) else {
            return;
        };
        let kind = MenuKind::Generic { rows: held.rows };
        let window = next_window(player);
        let mut open = PluginMenu {
            window,
            connection,
            inventory,
            menu: ContainerMenu::new(kind),
            state_id: 0,
        };
        player.link.send(ClientEvent::OpenScreen {
            window,
            menu: kind.protocol_id(),
            title: held.title.clone(),
        });
        sync_menu_view(
            &player.link,
            window,
            &mut open.state_id,
            &mut open.menu,
            &held.items,
            &player.inventory,
            true,
        );
        player.plugin_menu = Some(open);
    }

    pub(crate) fn handle_plugin_menu_click(
        &mut self,
        id: PlayerId,
        click: &ContainerClick,
    ) -> bool {
        let Some(player) = self.players.get_mut(&id) else {
            return false;
        };
        let Some(open) = player.plugin_menu.as_mut() else {
            return false;
        };
        if open.window != click.container {
            return true;
        }
        let key = open.key();
        let Some(held) = self.plugin_inventories.held.get_mut(&key) else {
            return true;
        };
        let size = held.items.len();
        let total = open.menu.kind().slot_count() as i16;
        if click.slot != NO_SLOT && click.slot != OUTSIDE && !(0..total).contains(&click.slot) {
            return true;
        }
        let slot = clicked_slot(click.slot, size);
        let item = view(slot, &held.items, &player.inventory)
            .as_ref()
            .map(spec_of);
        let refused = held.locked || player.game_mode == GameMode::Spectator;
        let effects = if refused {
            Vec::new()
        } else {
            let clicker = Clicker {
                infinite_materials: player.game_mode == GameMode::Creative,
            };
            open.menu.click(
                &mut held.items,
                &mut player.inventory,
                click.slot,
                click.button,
                click.input,
                clicker,
            )
        };
        for (slot, hash) in &click.changed {
            open.menu.receive_remote(*slot, hash.clone());
        }
        open.menu.receive_remote_carried(click.carried.clone());
        let full =
            refused || click.state_id != open.state_id || effects.contains(&MenuEffect::FullResync);
        let drops = effects
            .into_iter()
            .filter(|effect| matches!(effect, MenuEffect::Drop { .. }))
            .collect();
        self.apply_menu_effects(id, drops);
        self.sync_plugin_viewer(id, full);
        if refused && click.input == ClickInput::Swap && click.button == OFFHAND_BUTTON {
            self.resend_offhand(id);
        }
        if !refused {
            for viewer in self.plugin_viewers(|menu| menu.key() == key) {
                if viewer != id {
                    self.sync_plugin_viewer(viewer, false);
                }
            }
        }
        self.sync_inventory(id);
        if let (Some(kind), Some(player)) =
            (click_kind(click.input, click.button), self.snapshot_of(id))
        {
            self.notify_owner(
                key.0,
                EventPayload::InventoryClick {
                    player,
                    inventory: key.1,
                    slot,
                    kind,
                    item,
                },
            );
        }
        true
    }

    pub(crate) fn close_plugin_menu(
        &mut self,
        id: PlayerId,
        notify_client: bool,
        by_player: bool,
    ) -> bool {
        let Some(player) = self.players.get_mut(&id) else {
            return false;
        };
        let Some(mut open) = player.plugin_menu.take() else {
            return false;
        };
        let before = Self::equipment_of(player);
        let effects = open.menu.close(&mut player.inventory);
        if notify_client {
            player.link.send(ClientEvent::ContainerClose(open.window));
        }
        self.apply_menu_effects(id, effects);
        self.broadcast_equipment_changes(id, before);
        if let Some(player) = self.snapshot_of(id) {
            self.notify_owner(
                open.connection,
                EventPayload::InventoryClose {
                    player,
                    inventory: open.inventory,
                    by_player,
                },
            );
        }
        true
    }

    pub(crate) fn forget_plugin_inventories(&mut self, connection: u64) {
        for viewer in self.plugin_viewers(|menu| menu.connection == connection) {
            self.close_plugin_menu(viewer, true, false);
        }
        self.plugin_inventories
            .held
            .retain(|(owner, _), _| *owner != connection);
    }

    pub(super) fn apply_custom_inventory_order(
        &mut self,
        connection: u64,
        inventory: u64,
        order: CustomInventoryOrder,
    ) {
        let key = (connection, inventory);
        if !self.plugin_inventories.held.contains_key(&key) {
            return;
        }
        let name = order.name();
        if let Err(reason) = self.custom_inventory_order(key, order) {
            self.warn_plugin(connection, name, reason);
        }
    }

    fn custom_inventory_order(
        &mut self,
        key: Key,
        order: CustomInventoryOrder,
    ) -> Result<(), String> {
        match order {
            CustomInventoryOrder::Set { slot, item } => {
                let Some(held) = self.plugin_inventories.held.get_mut(&key) else {
                    return Ok(());
                };
                let size = held.items.len();
                let index = usize::try_from(slot)
                    .ok()
                    .filter(|index| *index < size)
                    .ok_or_else(|| format!("slot {slot} is outside 0..={}", size - 1))?;
                held.items[index] = match item {
                    Some(spec) => stack_of(&spec)?,
                    None => None,
                };
                self.sync_plugin_viewers(key);
            }
            CustomInventoryOrder::Clear => {
                if let Some(held) = self.plugin_inventories.held.get_mut(&key) {
                    held.items.fill(None);
                }
                self.sync_plugin_viewers(key);
            }
            CustomInventoryOrder::Close => self.close_plugin_viewers(key),
            CustomInventoryOrder::Discard => {
                self.close_plugin_viewers(key);
                self.plugin_inventories.held.remove(&key);
            }
        }
        Ok(())
    }

    pub(super) fn custom_inventory_state(
        &self,
        connection: u64,
        inventory: u64,
    ) -> Option<CustomInventoryState> {
        let key = (connection, inventory);
        let held = self.plugin_inventories.held.get(&key)?;
        let mut viewers: Vec<PlayerSnapshot> = self
            .plugin_viewers(|menu| menu.key() == key)
            .into_iter()
            .filter_map(|viewer| self.snapshot_of(viewer))
            .collect();
        viewers.sort_by_key(|viewer| viewer.entity_id);
        Some(CustomInventoryState {
            items: held
                .items
                .iter()
                .map(|slot| slot.as_ref().map(spec_of))
                .collect(),
            viewers,
        })
    }

    fn plugin_viewers(&self, viewing: impl Fn(&PluginMenu) -> bool) -> Vec<PlayerId> {
        self.players
            .iter()
            .filter(|(_, player)| player.plugin_menu.as_ref().is_some_and(&viewing))
            .map(|(id, _)| *id)
            .collect()
    }

    fn sync_plugin_viewers(&mut self, key: Key) {
        for viewer in self.plugin_viewers(|menu| menu.key() == key) {
            self.sync_plugin_viewer(viewer, false);
        }
    }

    fn close_plugin_viewers(&mut self, key: Key) {
        for viewer in self.plugin_viewers(|menu| menu.key() == key) {
            self.close_plugin_menu(viewer, true, false);
        }
    }

    fn sync_plugin_viewer(&mut self, id: PlayerId, full: bool) {
        let Some(player) = self.players.get_mut(&id) else {
            return;
        };
        let Some(open) = player.plugin_menu.as_mut() else {
            return;
        };
        let Some(held) = self.plugin_inventories.held.get(&open.key()) else {
            return;
        };
        sync_menu_view(
            &player.link,
            open.window,
            &mut open.state_id,
            &mut open.menu,
            &held.items,
            &player.inventory,
            full,
        );
    }

    fn resend_offhand(&self, id: PlayerId) {
        if let Some(player) = self.players.get(&id) {
            player.link.send(ClientEvent::PlayerInventorySlot {
                slot: OFFHAND,
                stack: player.inventory.get(OFFHAND_SLOT).cloned(),
            });
        }
    }
}

fn clicked_slot(slot: i16, size: usize) -> ClickedSlot {
    let Ok(index) = usize::try_from(slot) else {
        return ClickedSlot::Outside;
    };
    if index < size {
        ClickedSlot::Menu(index as u16)
    } else if index < size + MAIN_SLOTS {
        ClickedSlot::Player(FIRST_MAIN + (index - size) as i32)
    } else {
        ClickedSlot::Player((index - size - MAIN_SLOTS) as i32)
    }
}

fn view(slot: ClickedSlot, items: &[Slot], inventory: &Inventory) -> Slot {
    match slot {
        ClickedSlot::Menu(index) => items.get(usize::from(index)).cloned().flatten(),
        ClickedSlot::Player(spigot) => usize::try_from(spigot)
            .ok()
            .and_then(menu_slot)
            .and_then(|index| inventory.get(index))
            .cloned(),
        ClickedSlot::Outside => None,
    }
}

fn click_kind(input: ClickInput, button: i8) -> Option<ClickKind> {
    match (input, button) {
        (ClickInput::Pickup, 0) => Some(ClickKind::Left),
        (ClickInput::Pickup, 1) => Some(ClickKind::Right),
        (ClickInput::QuickMove, 0) => Some(ClickKind::ShiftLeft),
        (ClickInput::QuickMove, 1) => Some(ClickKind::ShiftRight),
        (ClickInput::Swap, OFFHAND_BUTTON) => Some(ClickKind::SwapOffhand),
        (ClickInput::Swap, key) => u8::try_from(key)
            .ok()
            .filter(|key| *key < 9)
            .map(ClickKind::NumberKey),
        (ClickInput::Clone, _) => Some(ClickKind::Middle),
        (ClickInput::Throw, 0) => Some(ClickKind::Drop),
        (ClickInput::Throw, 1) => Some(ClickKind::ControlDrop),
        (ClickInput::PickupAll, _) => Some(ClickKind::DoubleClick),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn menu_slots_map_to_the_menu_then_the_main_inventory_then_the_hotbar() {
        assert_eq!(clicked_slot(OUTSIDE, 9), ClickedSlot::Outside);
        assert_eq!(clicked_slot(NO_SLOT, 9), ClickedSlot::Outside);
        assert_eq!(clicked_slot(0, 9), ClickedSlot::Menu(0));
        assert_eq!(clicked_slot(53, 54), ClickedSlot::Menu(53));
        assert_eq!(clicked_slot(9, 9), ClickedSlot::Player(9));
        assert_eq!(clicked_slot(35, 9), ClickedSlot::Player(35));
        assert_eq!(clicked_slot(36, 9), ClickedSlot::Player(0));
        assert_eq!(clicked_slot(44, 9), ClickedSlot::Player(8));
        assert_eq!(clicked_slot(54, 54), ClickedSlot::Player(9));
        assert_eq!(clicked_slot(89, 54), ClickedSlot::Player(8));
    }

    #[test]
    fn clicks_map_to_their_spigot_kind() {
        assert_eq!(
            click_kind(ClickInput::Swap, 0),
            Some(ClickKind::NumberKey(0))
        );
        assert_eq!(
            click_kind(ClickInput::Swap, 8),
            Some(ClickKind::NumberKey(8))
        );
        assert_eq!(click_kind(ClickInput::Swap, 9), None);
        assert_eq!(click_kind(ClickInput::Swap, -1), None);
        assert_eq!(
            click_kind(ClickInput::Swap, 40),
            Some(ClickKind::SwapOffhand)
        );
        assert_eq!(click_kind(ClickInput::Pickup, 2), None);
        assert_eq!(click_kind(ClickInput::QuickCraft, 0), None);
        assert_eq!(
            click_kind(ClickInput::Throw, 1),
            Some(ClickKind::ControlDrop)
        );
    }
}
