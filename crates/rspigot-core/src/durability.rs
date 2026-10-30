use rspigot_domain::entity::reward::{damage_value, max_damage, set_damage_value};
use rspigot_domain::item::{HOTBAR_START, OFFHAND_SLOT};
use rspigot_domain::player::{GameMode, PlayerId};

use crate::events::{ClientEvent, EntityStatus, EquipmentSlot, Hand};
use crate::ports::ClientLink;
use crate::server::{Audience, Server};

impl<L: ClientLink> Server<L> {
    pub(crate) fn hurt_and_break(&mut self, id: PlayerId, hand: Hand, amount: i32) {
        let Some(player) = self.players.get_mut(&id) else {
            return;
        };
        if player.game_mode == GameMode::Creative || amount <= 0 {
            return;
        }
        let (slot, equipment) = match hand {
            Hand::Main => (
                HOTBAR_START + usize::from(player.inventory.selected_hotbar),
                EquipmentSlot::MainHand,
            ),
            Hand::Off => (OFFHAND_SLOT, EquipmentSlot::OffHand),
        };
        let Some(mut stack) = player.inventory.get(slot).cloned() else {
            return;
        };
        let Some(max) = max_damage(&stack) else {
            return;
        };
        let before = Self::equipment_of(player);
        let damage = damage_value(&stack) + amount;
        set_damage_value(&mut stack, damage);
        let broken = damage_value(&stack) >= max;
        let entity = player.entity_id;
        player
            .inventory
            .set(slot, if broken { None } else { Some(stack) });
        if broken {
            let status = ClientEvent::EntityStatus {
                entity,
                status: EntityStatus::EquipmentBreak(equipment),
            };
            self.push(Audience::One(id), status.clone());
            self.push(Audience::Trackers(entity), status);
        }
        self.sync_inventory(id);
        self.broadcast_equipment_changes(id, before);
    }

    pub(crate) fn wear_tool_for_mining(&mut self, id: PlayerId, hardness: f32) {
        if hardness == 0.0 {
            return;
        }
        let amount = self
            .players
            .get(&id)
            .and_then(|player| player.inventory.held())
            .and_then(|stack| stack.item.kind())
            .and_then(|kind| kind.tool())
            .and_then(|tool| i32::try_from(tool.damage_per_block).ok());
        if let Some(amount) = amount {
            self.hurt_and_break(id, Hand::Main, amount);
        }
    }
}
