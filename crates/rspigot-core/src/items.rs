use rand::Rng;
use rspigot_blocks::StateFlags;
use rspigot_domain::item::ItemStack;
use rspigot_domain::{BlockPos, EntityId, Rotation, Vec3};
use uuid::Uuid;

use crate::events::{ClientEvent, EntityKind, MetadataEntry, MetadataValue};
use crate::ports::ClientLink;
use crate::server::{Audience, Server};

const GRAVITY: f64 = 0.04;
const DRAG: f64 = 0.98;
const BOUNCE: f64 = -0.5;
const PICKUP_DELAY: u16 = 10;
const ITEM_HALF_WIDTH: f64 = 0.125;
const ITEM_HEIGHT: f64 = 0.25;
const PLAYER_HALF_WIDTH: f64 = 0.3;
const PLAYER_HEIGHT: f64 = 1.8;
const ITEM_DATA_INDEX: u8 = 8;
const POSITION_SYNC_TICKS: u32 = 20;

#[derive(Debug, Clone, PartialEq)]
pub struct ItemEntity {
    pub id: EntityId,
    pub uuid: Uuid,
    pub stack: ItemStack,
    pub position: Vec3,
    pub velocity: Vec3,
    pub on_ground: bool,
    pub pickup_delay: u16,
    pub age: u32,
    synced_position: Vec3,
}

impl ItemEntity {
    pub fn spawn_event(&self) -> ClientEvent {
        ClientEvent::SpawnEntity {
            entity: self.id,
            uuid: self.uuid,
            kind: EntityKind::Item,
            position: self.position,
            rotation: Rotation::default(),
            head_yaw: 0.0,
            data: 0,
            velocity: self.velocity,
        }
    }

    pub fn metadata_event(&self) -> ClientEvent {
        ClientEvent::EntityMetadata {
            entity: self.id,
            entries: vec![MetadataEntry {
                index: ITEM_DATA_INDEX,
                value: MetadataValue::Item(Some(self.stack.clone())),
            }],
        }
    }

    fn touches_player(&self, player: Vec3) -> bool {
        let reach_x = PLAYER_HALF_WIDTH + 1.0 + ITEM_HALF_WIDTH;
        let below = player.y - 0.5 - ITEM_HEIGHT;
        let above = player.y + PLAYER_HEIGHT + 0.5;
        (self.position.x - player.x).abs() <= reach_x
            && (self.position.z - player.z).abs() <= reach_x
            && self.position.y >= below
            && self.position.y <= above
    }
}

impl<L: ClientLink> Server<L> {
    pub(crate) fn spawn_item(&mut self, pos: BlockPos, stack: ItemStack) {
        let id = self.allocate_entity_id();
        let offset = |rng: &mut rand::rngs::StdRng| rng.random_range(-0.25..0.25);
        let position = Vec3::new(
            f64::from(pos.x) + 0.5 + offset(&mut self.rng),
            f64::from(pos.y) + 0.5 + offset(&mut self.rng) - ITEM_HEIGHT / 2.0,
            f64::from(pos.z) + 0.5 + offset(&mut self.rng),
        );
        let velocity = Vec3::new(
            self.rng.random::<f64>() * 0.2 - 0.1,
            0.2,
            self.rng.random::<f64>() * 0.2 - 0.1,
        );
        let uuid = uuid::Builder::from_random_bytes(self.rng.random()).into_uuid();
        self.items.insert(
            id,
            ItemEntity {
                id,
                uuid,
                stack,
                position,
                velocity,
                on_ground: false,
                pickup_delay: PICKUP_DELAY,
                age: 0,
                synced_position: position,
            },
        );
    }

    fn collides(&self, x: f64, y: f64, z: f64) -> bool {
        let pos = Vec3::new(x, y, z).block();
        self.world
            .chunks
            .block(pos)
            .is_some_and(|state| state.flags().contains(StateFlags::HAS_COLLISION))
    }

    fn friction_below(&self, position: Vec3) -> f64 {
        let below = Vec3::new(position.x, position.y - 0.5, position.z).block();
        self.world
            .chunks
            .block(below)
            .and_then(|state| state.kind())
            .map_or(0.6, |kind| f64::from(kind.settings().friction))
    }

    pub(crate) fn tick_items(&mut self) {
        let despawn_after = self.config.item_despawn_ticks;
        let ids: Vec<EntityId> = self.items.keys().copied().collect();
        let mut removed = Vec::new();
        for id in ids {
            let Some(mut item) = self.items.remove(&id) else {
                continue;
            };
            item.age += 1;
            item.pickup_delay = item.pickup_delay.saturating_sub(1);
            if item.age >= despawn_after || !self.world.chunks.is_loaded(item.position.chunk()) {
                removed.push(id);
                continue;
            }
            self.step_item(&mut item);
            let moved = item.position.distance_squared(item.synced_position) > 1.0e-6;
            if moved && (item.age.is_multiple_of(POSITION_SYNC_TICKS) || item.on_ground) {
                self.push(
                    Audience::Trackers(id),
                    ClientEvent::EntityMoved {
                        entity: id,
                        from: item.synced_position,
                        to: item.position,
                        rotation: None,
                        on_ground: item.on_ground,
                    },
                );
                item.synced_position = item.position;
            }
            self.items.insert(id, item);
        }
        for id in removed {
            self.remove_item(id);
        }
        self.collect_items();
    }

    fn step_item(&self, item: &mut ItemEntity) {
        if item.on_ground && item.velocity.x.abs() < 1.0e-3 && item.velocity.z.abs() < 1.0e-3 {
            item.velocity = Vec3::ZERO;
            if self.collides(item.position.x, item.position.y - 0.01, item.position.z) {
                return;
            }
            item.on_ground = false;
        }
        item.velocity.y -= GRAVITY;
        let mut next = item.position + item.velocity;
        if self.collides(next.x, item.position.y + 0.01, item.position.z) {
            next.x = item.position.x;
            item.velocity.x = 0.0;
        }
        if self.collides(next.x, item.position.y + 0.01, next.z) {
            next.z = item.position.z;
            item.velocity.z = 0.0;
        }
        item.on_ground = false;
        if item.velocity.y < 0.0 && self.collides(next.x, next.y, next.z) {
            next.y = next.y.floor() + 1.0;
            item.on_ground = true;
        } else if item.velocity.y > 0.0 && self.collides(next.x, next.y + ITEM_HEIGHT, next.z) {
            next.y = item.position.y;
            item.velocity.y = 0.0;
        }
        item.position = next;
        let horizontal = if item.on_ground {
            self.friction_below(item.position) * DRAG
        } else {
            DRAG
        };
        item.velocity = Vec3::new(
            item.velocity.x * horizontal,
            item.velocity.y * DRAG,
            item.velocity.z * horizontal,
        );
        if item.on_ground && item.velocity.y < 0.0 {
            item.velocity.y *= BOUNCE;
            if item.velocity.y < GRAVITY {
                item.velocity.y = 0.0;
            }
        }
    }

    fn collect_items(&mut self) {
        let collectors: Vec<(rspigot_domain::player::PlayerId, EntityId, Vec3)> = self
            .players
            .values()
            .filter(|p| {
                p.game_mode != rspigot_domain::player::GameMode::Spectator
                    && p.pending_teleport.is_none()
            })
            .map(|p| (p.id, p.entity_id, p.position))
            .collect();
        for (player_id, collector, position) in collectors {
            let touching: Vec<EntityId> = self
                .items
                .values()
                .filter(|item| item.pickup_delay == 0 && item.touches_player(position))
                .map(|item| item.id)
                .collect();
            for id in touching {
                self.pick_up(player_id, collector, id);
            }
        }
    }

    fn pick_up(
        &mut self,
        player_id: rspigot_domain::player::PlayerId,
        collector: EntityId,
        id: EntityId,
    ) {
        let Some(item) = self.items.get(&id) else {
            return;
        };
        let stack = item.stack.clone();
        let Some(player) = self.players.get_mut(&player_id) else {
            return;
        };
        let outcome = player.inventory.insert(
            stack.item,
            u32::from(stack.count.get()),
            stack.item.max_stack_size(),
        );
        let taken = u32::from(stack.count.get()) - outcome.remaining;
        if taken == 0 {
            return;
        }
        for slot in outcome.changed {
            let state_id = player.next_inventory_state();
            player.link.send(ClientEvent::InventorySlot {
                state_id,
                slot: slot as i16,
                stack: player.inventory.get(slot).cloned(),
            });
        }
        let pickup = ClientEvent::ItemPickup {
            collected: id,
            collector,
            amount: taken as i32,
        };
        player.link.send(pickup.clone());
        self.push(Audience::TrackersExcept(id, player_id), pickup);
        self.broadcast_held_item(player_id);
        match std::num::NonZeroU8::new(outcome.remaining as u8) {
            Some(count) => {
                if let Some(item) = self.items.get_mut(&id) {
                    item.stack.count = count;
                    let metadata = item.metadata_event();
                    self.push(Audience::Trackers(id), metadata);
                }
            }
            None => self.remove_item(id),
        }
    }

    pub(crate) fn remove_item(&mut self, id: EntityId) {
        self.items.remove(&id);
        for player in self.players.values_mut() {
            if player.tracked.remove(&id) {
                player.link.send(ClientEvent::RemoveEntities(vec![id]));
            }
        }
    }

    pub(crate) fn update_item_tracking(&mut self) {
        let range = self.config.item_tracking_range * self.config.item_tracking_range;
        for viewer in self.players.values_mut() {
            for item in self.items.values() {
                let visible = viewer.position.distance_squared(item.position) <= range
                    && viewer.streamer.has_sent(item.position.chunk());
                let tracking = viewer.tracked.contains(&item.id);
                if visible && !tracking {
                    viewer.tracked.insert(item.id);
                    viewer.link.send(item.spawn_event());
                    viewer.link.send(item.metadata_event());
                    viewer.link.send(ClientEvent::EntityVelocity {
                        entity: item.id,
                        velocity: item.velocity,
                    });
                } else if !visible && tracking {
                    viewer.tracked.remove(&item.id);
                    viewer.link.send(ClientEvent::RemoveEntities(vec![item.id]));
                }
            }
        }
    }

    pub fn items_on_ground(&self) -> usize {
        self.items.len()
    }
}
