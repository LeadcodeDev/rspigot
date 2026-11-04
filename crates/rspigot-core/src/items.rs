use rand::Rng;
use rspigot_domain::entity::{Entity, EntityState, FLAG_ON_FIRE, ItemState, RemovalReason, item};
use rspigot_domain::item::ItemStack;
use rspigot_domain::player::{GameMode, PlayerId};
use rspigot_domain::{BlockPos, EntityId, Vec3};
use rspigot_entities::EntityKind as Kind;

use crate::events::ClientEvent;
use crate::plugins::events::{CreatureSpawnReason, EntityRemoveCause};
use crate::plugins::steps::Announced;
use crate::plugins::{EntityView, view_of_entity};
use crate::ports::ClientLink;
use crate::server::{Audience, Server};

pub(crate) const PICKUP_DELAY: u16 = 10;
const ITEM_HALF_WIDTH: f64 = 0.125;
const ITEM_HEIGHT: f64 = 0.25;
const PLAYER_HALF_WIDTH: f64 = 0.3;
const PLAYER_HEIGHT: f64 = 1.8;
const BELOW_WORLD: i32 = 64;

pub(crate) fn launched(launch: Announced, stack: ItemStack) -> Entity {
    let Announced {
        mut base,
        velocity,
        yaw,
    } = launch;
    base.set_yaw(yaw);
    base.velocity = velocity;
    let state = ItemState::new(&mut base, stack, i32::from(PICKUP_DELAY));
    Entity::new(base, EntityState::Item(state))
}

fn touches_player(position: Vec3, player: Vec3) -> bool {
    let reach_x = PLAYER_HALF_WIDTH + 1.0 + ITEM_HALF_WIDTH;
    let below = player.y - 0.5 - ITEM_HEIGHT;
    let above = player.y + PLAYER_HEIGHT + 0.5;
    (position.x - player.x).abs() <= reach_x
        && (position.z - player.z).abs() <= reach_x
        && position.y >= below
        && position.y <= above
}

impl<L: ClientLink> Server<L> {
    pub(crate) fn spawn_item(&mut self, pos: BlockPos, stack: ItemStack) {
        let launch = self.block_drop_launch(pos);
        self.drop_launched(launch, stack);
    }

    pub(crate) fn block_drop_launch(&mut self, pos: BlockPos) -> Announced {
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
        let mut base = self.new_entity_base(Kind::Item);
        base.position = position;
        base.old_position = position;
        let yaw = self.rng.random::<f32>() * 360.0;
        Announced {
            base,
            velocity,
            yaw,
        }
    }

    pub(crate) fn drop_launched(&mut self, launch: Announced, stack: ItemStack) {
        self.add_entity(launched(launch, stack), CreatureSpawnReason::Default);
    }

    pub(crate) fn spawn_item_from_face(
        &mut self,
        pos: BlockPos,
        step: (i32, i32, i32),
        stack: ItemStack,
    ) {
        let (dx, dy, dz) = step;
        let half_height = ITEM_HEIGHT / 2.0;
        let mut between = |low: f64, high: f64| low + self.rng.random::<f64>() * (high - low);
        let spread = |step: i32, between: &mut dyn FnMut(f64, f64) -> f64, reach: f64| {
            if step == 0 {
                between(-0.25, 0.25)
            } else {
                f64::from(step) * (0.5 + reach)
            }
        };
        let position = Vec3::new(
            f64::from(pos.x) + 0.5 + spread(dx, &mut between, ITEM_HALF_WIDTH),
            f64::from(pos.y) + 0.5 + spread(dy, &mut between, half_height) - half_height,
            f64::from(pos.z) + 0.5 + spread(dz, &mut between, ITEM_HALF_WIDTH),
        );
        let velocity = Vec3::new(
            if dx == 0 {
                between(-0.1, 0.1)
            } else {
                f64::from(dx) * 0.1
            },
            if dy == 0 {
                between(0.0, 0.1)
            } else {
                f64::from(dy) * 0.1 + 0.1
            },
            if dz == 0 {
                between(-0.1, 0.1)
            } else {
                f64::from(dz) * 0.1
            },
        );
        self.spawn_item_entity(position, velocity, PICKUP_DELAY, stack);
    }

    pub(crate) fn spawn_item_entity(
        &mut self,
        position: Vec3,
        velocity: Vec3,
        pickup_delay: u16,
        stack: ItemStack,
    ) -> EntityView {
        let item = self.item_entity(position, velocity, pickup_delay, stack);
        let view = view_of_entity(&item);
        self.add_entity(item, CreatureSpawnReason::Default);
        view
    }

    pub(crate) fn item_entity(
        &mut self,
        position: Vec3,
        velocity: Vec3,
        pickup_delay: u16,
        stack: ItemStack,
    ) -> Entity {
        let mut base = self.new_entity_base(Kind::Item);
        base.position = position;
        base.old_position = position;
        let yaw = self.rng.random::<f32>() * 360.0;
        base.set_yaw(yaw);
        base.velocity = velocity;
        let state = ItemState::new(&mut base, stack, i32::from(pickup_delay));
        Entity::new(base, EntityState::Item(state))
    }

    pub(crate) fn tick_item_entity(&mut self, id: EntityId) {
        let despawn_after = i64::from(self.config.item_despawn_ticks);
        let floor = f64::from(self.world.dimension.height.min_y - BELOW_WORLD);
        let chunks = &self.world.chunks;
        let rng = &mut self.rng;
        let Some(Entity {
            base,
            state: EntityState::Item(state),
            ..
        }) = self.entities.store.get_mut(id)
        else {
            return;
        };
        let mut discard = item::stack(base)
            .is_none()
            .then_some(EntityRemoveCause::Despawn);
        if discard.is_none() {
            base.old_position = base.position;
            base.tick_count = base.tick_count.wrapping_add(1);
            if base.remaining_fire_ticks > 0 {
                base.remaining_fire_ticks = if base.kind.fire_immune() {
                    0
                } else {
                    base.remaining_fire_ticks - 1
                };
            }
            base.set_shared_flag(FLAG_ON_FIRE, base.remaining_fire_ticks > 0);
            if base.position.y < floor {
                discard = Some(EntityRemoveCause::OutOfWorld);
            } else {
                let mut next_float = || rng.random::<f32>();
                item::tick_item(base, state, chunks, &mut next_float);
                discard =
                    (i64::from(state.age) >= despawn_after).then_some(EntityRemoveCause::Despawn);
            }
        }
        if let Some(cause) = discard {
            self.remove_entity(id, RemovalReason::Discarded, Some(cause));
        }
    }

    pub(crate) fn collect_items(&mut self) {
        let collectors: Vec<(PlayerId, EntityId, Vec3)> = self
            .players
            .values()
            .filter(|p| {
                p.game_mode != GameMode::Spectator
                    && p.pending_teleport.is_none()
                    && !p.vitals.is_dead()
            })
            .map(|p| (p.id, p.entity_id, p.position))
            .collect();
        for (player_id, collector, position) in collectors {
            let touching: Vec<EntityId> = self
                .entities
                .store
                .iter()
                .filter(|entity| match &entity.state {
                    EntityState::Item(state) => {
                        state.pickup_delay < 1 && touches_player(entity.base.position, position)
                    }
                    _ => false,
                })
                .map(|entity| entity.base.id)
                .collect();
            for id in touching {
                self.pick_up(player_id, collector, id);
            }
        }
    }

    fn pick_up(&mut self, player_id: PlayerId, collector: EntityId, id: EntityId) {
        let Some(stack) = self.item_stack_of(id) else {
            return;
        };
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
        let pickup = ClientEvent::ItemPickup {
            collected: id,
            collector,
            amount: taken as i32,
        };
        player.link.send(pickup.clone());
        self.sync_inventory(player_id);
        self.push(Audience::TrackersExcept(id, player_id), pickup);
        self.broadcast_held_item(player_id);
        match std::num::NonZeroU8::new(outcome.remaining as u8) {
            Some(count) => {
                if let Some(entity) = self.entities.store.get_mut(id) {
                    let mut stack = stack;
                    stack.count = count;
                    item::set_stack(&mut entity.base, Some(stack));
                }
            }
            None => self.remove_item(id),
        }
    }

    pub(crate) fn remove_item(&mut self, id: EntityId) {
        self.remove_entity(
            id,
            RemovalReason::Discarded,
            Some(EntityRemoveCause::Pickup),
        );
    }

    pub fn ground_items(&self) -> Vec<(EntityId, ItemStack)> {
        self.entities
            .store
            .iter()
            .filter_map(|e| item::stack(&e.base).map(|stack| (e.base.id, stack.clone())))
            .collect()
    }

    pub fn items_on_ground(&self) -> usize {
        self.entities
            .store
            .count_where(|e| matches!(e.state, EntityState::Item(_)))
    }
}
