use rand::Rng;
use rspigot_domain::Vec3;
use rspigot_domain::entity::item::{THROWN_DROP_BELOW_EYES, THROWN_PICKUP_DELAY, thrown_velocity};
use rspigot_domain::entity::{Entity, EntityState, ItemState};
use rspigot_domain::item::{HOTBAR_START, ItemStack, Slot, split};
use rspigot_domain::player::{GameMode, PlayerId};
use rspigot_domain::text::Component;
use rspigot_entities::EntityKind as Kind;
use uuid::Uuid;

use crate::events::{Animation, ClientEvent};
use crate::ports::ClientLink;
use crate::server::{Audience, Server};

const DROPS_PER_TICK: u32 = 20;
const CREATIVE_DROP_STEP: u32 = 20;
const CREATIVE_DROP_THRESHOLD: u32 = 1480;
const STANDING_EYES: f32 = 1.62;
const CROUCHING_EYES: f32 = 1.27;

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct DropLimits {
    creative: u32,
    last_tick: u64,
    this_tick: u32,
}

impl DropLimits {
    pub fn tick(&mut self) {
        self.creative = self.creative.saturating_sub(1);
    }

    fn allow_creative(&mut self) -> bool {
        if self.creative >= CREATIVE_DROP_THRESHOLD {
            return false;
        }
        self.creative += CREATIVE_DROP_STEP;
        true
    }

    fn allow_single(&mut self, tick: u64) -> bool {
        if self.last_tick != tick {
            self.this_tick = 0;
            self.last_tick = tick;
            return true;
        }
        self.this_tick += 1;
        self.this_tick < DROPS_PER_TICK
    }
}

impl<L: ClientLink> Server<L> {
    pub(crate) fn handle_drop_action(&mut self, id: PlayerId, all: bool) {
        let tick = self.current_tick();
        let Some(player) = self.players.get_mut(&id) else {
            return;
        };
        if player.vitals.is_dead()
            || !player.vitals.client_loaded()
            || player.game_mode == GameMode::Spectator
        {
            return;
        }
        if !all && !player.drops.allow_single(tick) {
            tracing::warn!("{} dropped their items too quickly!", player.profile.name);
            self.disconnect_player(
                id,
                Component::text("You dropped your items too quickly (Hacking?)"),
            );
            return;
        }
        self.drop_selected(id, all);
    }

    fn drop_selected(&mut self, id: PlayerId, all: bool) {
        let Some(player) = self.players.get_mut(&id) else {
            return;
        };
        let before = Self::equipment_of(player);
        let slot = HOTBAR_START + usize::from(player.inventory.selected_hotbar);
        let mut held: Slot = player.inventory.get(slot).cloned();
        let amount = if all { u32::MAX } else { 1 };
        let stack = split(&mut held, amount);
        player.inventory.set(slot, held.clone());
        player.menu.set_remote(slot, held);
        if let Some(stack) = stack {
            self.drop_from_player(id, stack, true, true);
        }
        if let Some(player) = self.players.get_mut(&id) {
            player.vitals.attack_ticker = 0;
        }
        self.broadcast_equipment_changes(id, before);
    }

    pub(crate) fn drop_creative(&mut self, id: PlayerId, stack: Slot) {
        let Some(player) = self.players.get_mut(&id) else {
            return;
        };
        if !player.drops.allow_creative() {
            tracing::warn!(
                "Player {} was dropping items too fast in creative mode, ignoring.",
                player.profile.name
            );
            return;
        }
        if let Some(stack) = stack {
            self.drop_from_player(id, stack, true, true);
        }
    }

    pub(crate) fn drop_from_player(
        &mut self,
        id: PlayerId,
        stack: ItemStack,
        thrower: bool,
        predicted: bool,
    ) {
        let Some(player) = self.players.get(&id) else {
            return;
        };
        let entity = player.entity_id;
        let eyes = if player.sneaking {
            CROUCHING_EYES
        } else {
            STANDING_EYES
        };
        let position = Vec3::new(
            player.position.x,
            player.position.y + f64::from(eyes) - f64::from(THROWN_DROP_BELOW_EYES),
            player.position.z,
        );
        let rotation = player.rotation;
        let owner = thrower.then_some(id.0);
        let swing = ClientEvent::EntityAnimation {
            entity,
            animation: Animation::SwingMainArm,
        };
        if !predicted {
            self.push(Audience::One(id), swing.clone());
        }
        self.push(Audience::Trackers(entity), swing);
        let yaw = self.rng.random::<f32>() * 360.0;
        let rolls: [f32; 4] = std::array::from_fn(|_| self.rng.random::<f32>());
        let velocity = thrown_velocity(rotation.yaw, rotation.pitch, rolls);
        self.spawn_thrown_item(position, velocity, yaw, stack, owner)
    }

    fn spawn_thrown_item(
        &mut self,
        position: Vec3,
        velocity: Vec3,
        yaw: f32,
        stack: ItemStack,
        thrower: Option<Uuid>,
    ) {
        let mut base = self.new_entity_base(Kind::Item);
        base.position = position;
        base.old_position = position;
        base.set_yaw(yaw);
        base.velocity = velocity;
        let mut state = ItemState::new(&mut base, stack, THROWN_PICKUP_DELAY);
        state.thrower = thrower;
        let _ = self.add_entity(Entity::new(base, EntityState::Item(state)));
    }
}
