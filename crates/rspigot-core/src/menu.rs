use rspigot_domain::item::{OFFHAND_SLOT, Slot, stacks_match};
use rspigot_domain::menu::{Clicker, MenuEffect, OUTSIDE, RESULT_SLOT, RemoteUpdate};
use rspigot_domain::player::{GameMode, PlayerId};

use crate::events::{ClickInput, ClientEvent, ContainerClick, EquipmentSlot};
use crate::player::Player;
use crate::ports::ClientLink;
use crate::server::{Audience, Server};

const INVENTORY_MENU: i32 = 0;
const MENU_SLOTS: i16 = 46;
const EQUIPMENT: [EquipmentSlot; 6] = [
    EquipmentSlot::MainHand,
    EquipmentSlot::OffHand,
    EquipmentSlot::Feet,
    EquipmentSlot::Legs,
    EquipmentSlot::Chest,
    EquipmentSlot::Head,
];

pub(crate) type Equipment = [Slot; 6];

impl<L: ClientLink> Server<L> {
    pub(crate) fn equipment_of(player: &Player<L>) -> Equipment {
        let inventory = &player.inventory;
        [
            inventory.held(),
            inventory.get(OFFHAND_SLOT),
            inventory.get(8),
            inventory.get(7),
            inventory.get(6),
            inventory.get(5),
        ]
        .map(|stack| stack.cloned())
    }

    pub(crate) fn broadcast_equipment_changes(&mut self, id: PlayerId, before: Equipment) {
        let Some(player) = self.players.get(&id) else {
            return;
        };
        let entity = player.entity_id;
        let after = Self::equipment_of(player);
        let slots: Vec<(EquipmentSlot, Slot)> = EQUIPMENT
            .iter()
            .zip(before.iter().zip(after))
            .filter(|(_, (old, new))| !stacks_match(old, new))
            .map(|(slot, (_, new))| (*slot, new))
            .collect();
        if !slots.is_empty() {
            self.push(
                Audience::Trackers(entity),
                ClientEvent::EntityEquipment { entity, slots },
            );
        }
    }

    pub(crate) fn handle_container_click(&mut self, id: PlayerId, click: ContainerClick) {
        if self.handle_open_menu_click(id, &click) {
            return;
        }
        let Some(player) = self.players.get_mut(&id) else {
            return;
        };
        if player.vitals.is_dead() || click.container != INVENTORY_MENU {
            return;
        }
        let slot = click.slot;
        if slot != -1 && slot != OUTSIDE && slot >= MENU_SLOTS {
            tracing::debug!(
                "Player {} clicked invalid slot index: {slot}, available slots: {MENU_SLOTS}",
                player.profile.name
            );
            return;
        }
        let full_resync = click.state_id != player.inventory_state;
        player.menu.suppress_remote_updates();
        if slot < -1 && slot != OUTSIDE {
            return;
        }
        let before = Self::equipment_of(player);
        let spectator = player.game_mode == GameMode::Spectator;
        let effects = if !spectator
            || click.input == ClickInput::QuickCraft
            || player.menu.quick_crafting()
        {
            let clicker = Clicker {
                infinite_materials: player.game_mode == GameMode::Creative,
            };
            player.menu.click(
                &mut player.inventory,
                slot,
                click.button,
                click.input,
                clicker,
            )
        } else {
            Vec::new()
        };
        self.apply_menu_effects(id, effects);
        let Some(player) = self.players.get_mut(&id) else {
            return;
        };
        for (slot, hash) in click.changed {
            if !player.menu.receive_remote(slot, hash) {
                tracing::debug!("Incorrect slot index: {slot} available slots: {MENU_SLOTS}");
            }
        }
        player.menu.receive_remote_carried(click.carried);
        player.menu.resume_remote_updates();
        if full_resync {
            self.send_inventory(id);
        } else {
            self.sync_inventory(id);
        }
        self.broadcast_equipment_changes(id, before);
    }

    pub(crate) fn handle_container_close(&mut self, id: PlayerId, window: i32) {
        if self.handle_open_menu_close(id, window) {
            return;
        }
        let Some(player) = self.players.get_mut(&id) else {
            return;
        };
        if player.vitals.is_dead() {
            return;
        }
        let before = Self::equipment_of(player);
        let effects = player.menu.close(&mut player.inventory);
        self.apply_menu_effects(id, effects);
        self.broadcast_equipment_changes(id, before);
    }

    pub(crate) fn set_slot_by_player(&mut self, id: PlayerId, slot: usize, stack: Slot) {
        let Some(player) = self.players.get_mut(&id) else {
            return;
        };
        let before = Self::equipment_of(player);
        let effects = player
            .menu
            .set_by_player(&mut player.inventory, slot, stack.clone());
        player.menu.set_remote(slot, stack);
        self.apply_menu_effects(id, effects);
        self.sync_inventory(id);
        self.broadcast_equipment_changes(id, before);
    }

    pub(crate) fn apply_menu_effects(&mut self, id: PlayerId, effects: Vec<MenuEffect>) {
        for effect in effects {
            match effect {
                MenuEffect::Drop {
                    stack,
                    thrower,
                    predicted,
                } => self.drop_from_player(id, stack, thrower, predicted),
                MenuEffect::ResultUpdated => {
                    if let Some(player) = self.players.get_mut(&id) {
                        let result = player.inventory.get(RESULT_SLOT).cloned();
                        player.menu.set_remote(RESULT_SLOT, result.clone());
                        Self::send_slot(player, RESULT_SLOT, result);
                    }
                }
                MenuEffect::SlotResent(slot) => {
                    if let Some(player) = self.players.get_mut(&id) {
                        for index in [slot, RESULT_SLOT] {
                            let stack = player.inventory.get(index).cloned();
                            Self::send_slot(player, index, stack);
                        }
                    }
                }
                MenuEffect::FurnaceOutputTaken => {}
                MenuEffect::FullResync => self.send_inventory(id),
                MenuEffect::PlacedBack { slot, stack } => {
                    if let Some(player) = self.players.get(&id) {
                        player.link.send(ClientEvent::PlayerInventorySlot {
                            slot: slot as i32,
                            stack: Some(stack),
                        });
                    }
                }
            }
        }
    }

    fn send_slot(player: &mut Player<L>, slot: usize, stack: Slot) {
        let state_id = player.next_inventory_state();
        player.link.send(ClientEvent::InventorySlot {
            state_id,
            slot: slot as i16,
            stack,
        });
    }

    pub(crate) fn sync_inventory(&mut self, id: PlayerId) {
        let Some(player) = self.players.get_mut(&id) else {
            return;
        };
        for update in player.menu.changes(&player.inventory) {
            match update {
                RemoteUpdate::Slot(slot, stack) => Self::send_slot(player, slot, stack),
                RemoteUpdate::Carried(stack) => player.link.send(ClientEvent::CursorItem(stack)),
            }
        }
    }

    pub(crate) fn tick_inventories(&mut self) {
        let ids: Vec<PlayerId> = self.players.keys().copied().collect();
        for id in ids {
            if let Some(player) = self.players.get_mut(&id) {
                player.drops.tick();
            }
            self.sync_inventory(id);
        }
    }
}
