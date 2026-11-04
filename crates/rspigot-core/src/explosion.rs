use rand::Rng;
use rspigot_domain::damage::DamageType;
use rspigot_domain::entity::species::{self, Species};
use rspigot_domain::entity::{Aabb, EntityState, RemovalReason, item};
use rspigot_domain::explosion::{self, MIN_RADIUS, PACKET_RANGE_SQR};
use rspigot_domain::item::{ItemStack, same_item_same_components};
use rspigot_domain::player::PlayerId;
use rspigot_domain::{BlockPos, BlockStateId, EntityId, Vec3};
use rspigot_entities::EntityKind as Kind;

use crate::events::ClientEvent;
use crate::health::DamageSource;
use crate::loot::{self, LootContext, LootEntity};
use crate::mobs::ServerRandom;
use crate::plugins::events::EntityRemoveCause;
use crate::plugins::steps::ExplodeStep;
use crate::plugins::{AfterHurt, BlastFollow, Holder, Struck};
use crate::ports::ClientLink;
use crate::server::{Audience, Server};

const MAX_MERGED_STACK: u32 = 16;
const EXPLOSION_KNOCKBACK_RESISTANCE: &str = "minecraft:explosion_knockback_resistance";
const POP_SPREAD: f64 = 0.25;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum BlockInteraction {
    Keep,
    Destroy,
    DestroyWithDecay,
}

struct HitEntity {
    id: EntityId,
    player: Option<PlayerId>,
    position: Vec3,
    eye: Vec3,
    bounds: Aabb,
    living: bool,
}

impl<L: ClientLink> Server<L> {
    pub(crate) fn explode_creeper(&mut self, id: EntityId) -> bool {
        let Some(entity) = self.entities.store.get_mut(id) else {
            return false;
        };
        let Some(Species::Creeper(creeper)) = entity
            .living()
            .and_then(|l| l.mob.as_ref())
            .map(|mob| mob.species)
        else {
            return false;
        };
        let multiplier = if species::is_powered(&entity.base) {
            2.0
        } else {
            1.0
        };
        let radius = creeper.explosion_radius as f32 * multiplier;
        let center = entity.base.position;
        if let Some(living) = entity.living_mut() {
            living.dead = true;
        }
        let Some(source) = self.mob_source(id) else {
            return false;
        };
        let damage = DamageSource::by(DamageType::PlayerExplosion, source);
        let interaction = if self.config.rules.mob_griefing {
            if self.config.rules.mob_explosion_drop_decay {
                BlockInteraction::DestroyWithDecay
            } else {
                BlockInteraction::Destroy
            }
        } else {
            BlockInteraction::Keep
        };
        self.explode(Some(id), &damage, center, radius, interaction);
        self.remove_entity(
            id,
            RemovalReason::Discarded,
            Some(EntityRemoveCause::Explode),
        );
        true
    }

    pub(crate) fn explode(
        &mut self,
        source: Option<EntityId>,
        damage: &DamageSource,
        center: Vec3,
        radius: f32,
        interaction: BlockInteraction,
    ) {
        let radius = radius.max(0.0);
        if radius < MIN_RADIUS {
            return;
        }
        let height = self.world.dimension.height;
        let positions = {
            let mut random = ServerRandom(&mut self.rng);
            explosion::exploded_positions(&self.world.chunks, height, center, radius, &mut random)
        };
        let knocked = self.hurt_by_explosion(source, damage, center, radius);
        let mut shuffled = positions;
        explosion::shuffle(&mut shuffled, &mut ServerRandom(&mut self.rng));
        let blocks: Vec<BlockPos> = shuffled
            .into_iter()
            .rev()
            .filter(|pos| {
                self.world
                    .chunks
                    .block(*pos)
                    .is_some_and(|state| !state.is_air())
            })
            .collect();
        let explosion_yield = if interaction == BlockInteraction::DestroyWithDecay {
            1.0 / radius
        } else {
            1.0
        };
        let step = ExplodeStep {
            center,
            radius,
            interaction,
            source,
            blocks,
            knocked,
            explosion_yield,
        };
        if source.is_none() && damage.causing.is_none() {
            self.ask_explosion(step);
        } else {
            self.blast(step);
        }
    }

    pub(crate) fn blast(&mut self, step: ExplodeStep) {
        let ExplodeStep {
            center,
            radius,
            interaction,
            source,
            blocks,
            knocked,
            explosion_yield,
        } = step;
        let silent = source
            .and_then(|id| self.entities.store.get(id))
            .is_some_and(|e| e.base.is_silent());
        let large = radius >= explosion::LARGE_RADIUS && interaction != BlockInteraction::Keep;
        let viewers: Vec<PlayerId> = self
            .players
            .values()
            .filter(|p| p.position.distance_squared(center) < PACKET_RANGE_SQR)
            .map(|p| p.id)
            .collect();
        for player in viewers {
            let knockback = knocked
                .iter()
                .find(|(id, _)| *id == player)
                .map(|(_, v)| *v);
            self.push(
                Audience::One(player),
                ClientEvent::Explode {
                    center,
                    radius,
                    block_count: blocks.len() as i32,
                    knockback,
                    large,
                    play_sound: !silent,
                },
            );
        }
        if interaction != BlockInteraction::Keep {
            let decay = (explosion_yield < 1.0).then(|| 1.0 / explosion_yield);
            self.blow_up_blocks(&blocks, source, decay);
        }
    }

    fn explosion_targets(&self, source: Option<EntityId>, area: &Aabb) -> Vec<HitEntity> {
        let mut out: Vec<HitEntity> = self
            .entities
            .store
            .iter()
            .filter(|e| Some(e.base.id) != source && !e.base.is_removed())
            .filter(|e| e.base.bounding_box().intersects(area))
            .filter(|e| e.is_simulated())
            .map(|e| HitEntity {
                id: e.base.id,
                player: None,
                position: e.base.position,
                eye: Vec3::new(e.base.position.x, e.base.eye_y(), e.base.position.z),
                bounds: e.base.bounding_box(),
                living: e.living().is_some(),
            })
            .collect();
        out.extend(
            self.players
                .values()
                .filter(|p| !p.vitals.is_dead())
                .filter_map(|p| {
                    let bounds = Aabb::of_size(p.position, 0.6, p.height() as f32);
                    bounds.intersects(area).then(|| HitEntity {
                        id: p.entity_id,
                        player: Some(p.id),
                        position: p.position,
                        eye: p.eye_position(),
                        bounds,
                        living: true,
                    })
                }),
        );
        out
    }

    fn hurt_by_explosion(
        &mut self,
        source: Option<EntityId>,
        damage: &DamageSource,
        center: Vec3,
        radius: f32,
    ) -> Vec<(PlayerId, Vec3)> {
        if radius < 1.0e-5 {
            return Vec::new();
        }
        let outer = self.impacts.knocked.replace(Vec::new());
        let area = explosion::hurt_area(center, radius);
        for target in self.explosion_targets(source, &area) {
            let distance =
                target.position.distance_squared(center).sqrt() / f64::from(radius * 2.0);
            if distance > 1.0 {
                continue;
            }
            let exposure = explosion::seen_percent(&self.world.chunks, center, &target.bounds);
            let amount = explosion::entity_damage(radius, center, target.position, exposure);
            let orb = target.player.is_none()
                && self
                    .entities
                    .store
                    .get(target.id)
                    .is_some_and(|e| matches!(e.state, EntityState::Orb(_)));
            if orb {
                self.damage_orb(target.id, amount);
            }
            let resistance = self
                .entities
                .store
                .get_mut(target.id)
                .and_then(|e| e.living_mut())
                .map_or(0.0, |living| {
                    living.attribute(EXPLOSION_KNOCKBACK_RESISTANCE)
                });
            let knockback = explosion::knockback(
                radius,
                center,
                target.position,
                target.eye,
                exposure,
                resistance,
            );
            let then = knockback.map_or(AfterHurt::Nothing, |knockback| {
                AfterHurt::Blast(BlastFollow {
                    target: target.id,
                    player: target.player,
                    knockback,
                    source,
                })
            });
            if orb {
                if let AfterHurt::Blast(follow) = then {
                    self.blast_knockback(follow);
                }
                continue;
            }
            let victim = match target.player {
                Some(player) => Struck::Player(player),
                None if target.living => Struck::Mob(target.id),
                None => Struck::Item(target.id),
            };
            self.strike(None, victim, damage.clone(), amount, then);
        }
        std::mem::replace(&mut self.impacts.knocked, outer).unwrap_or_default()
    }

    pub(crate) fn blast_knockback(&mut self, follow: BlastFollow) {
        let living = follow.player.is_some()
            || self
                .entities
                .store
                .get(follow.target)
                .is_some_and(|entity| entity.living().is_some());
        if living {
            let holder = follow
                .player
                .map_or(Holder::Mob(follow.target), Holder::Player);
            self.blast_push(holder, follow.knockback, follow.source);
            return;
        }
        if let Some(entity) = self.entities.store.get_mut(follow.target) {
            entity.base.velocity = entity.base.velocity + follow.knockback;
            entity.base.needs_sync = true;
        }
    }

    fn damage_orb(&mut self, id: EntityId, amount: f32) {
        let Some(entity) = self.entities.store.get_mut(id) else {
            return;
        };
        let destroyed = match &mut entity.state {
            EntityState::Orb(state) => {
                state.health -= amount.ceil() as i32;
                state.health <= 0
            }
            _ => false,
        };
        if destroyed {
            self.remove_entity(id, RemovalReason::Killed, Some(EntityRemoveCause::Death));
        }
    }

    fn blow_up_blocks(
        &mut self,
        blocks: &[BlockPos],
        source: Option<EntityId>,
        decay: Option<f32>,
    ) {
        let this = source
            .and_then(|id| self.entities.store.get(id))
            .map(|e| LootEntity::of(e.base.kind));
        let mut stacks: Vec<(BlockPos, ItemStack)> = Vec::new();
        for pos in blocks {
            let Some(state) = self.world.chunks.block(*pos) else {
                continue;
            };
            if state.is_air() {
                continue;
            }
            let context = LootContext {
                explosion_radius: decay,
                ..LootContext::block(state, None, this.as_ref())
            };
            let drops = loot::block_drops(&context, &mut self.rng);
            for drop in drops {
                merge_stack(&mut stacks, *pos, drop);
            }
            self.set_block(*pos, BlockStateId::AIR);
        }
        if !self.config.rules.block_drops {
            return;
        }
        for (pos, stack) in stacks {
            let x = f64::from(pos.x) + 0.5 + self.rng.random_range(-POP_SPREAD..=POP_SPREAD);
            let y =
                f64::from(pos.y) + 0.5 + self.rng.random_range(-POP_SPREAD..=POP_SPREAD) - 0.125;
            let z = f64::from(pos.z) + 0.5 + self.rng.random_range(-POP_SPREAD..=POP_SPREAD);
            let velocity = Vec3::new(
                self.rng.random::<f64>() * 0.2 - 0.1,
                0.2,
                self.rng.random::<f64>() * 0.2 - 0.1,
            );
            self.spawn_item_entity(Vec3::new(x, y, z), velocity, 10, stack);
        }
    }

    pub(crate) fn charged_creeper_drops(&mut self, killer: Option<EntityId>, victim: EntityId) {
        let Some(killer) = killer else {
            return;
        };
        let Some(entity) = self.entities.store.get(killer) else {
            return;
        };
        let powered = entity.base.kind == Kind::Creeper && species::is_powered(&entity.base);
        let dropped = matches!(
            entity.living().and_then(|l| l.mob.as_ref()).map(|m| m.species),
            Some(Species::Creeper(c)) if c.dropped_skulls
        );
        if !powered || dropped || !self.config.rules.mob_drops {
            return;
        }
        let Some(table) = rspigot_data::loot::table("minecraft:charged_creeper/root") else {
            return;
        };
        let Some(victim_entity) = self.entities.store.get(victim) else {
            return;
        };
        let position = victim_entity.base.position;
        let facts = crate::mobs::loot_facts_of(victim_entity);
        let context = LootContext {
            this: Some(&facts),
            ..LootContext::default()
        };
        let drops = loot::table_drops(table, &context, &mut self.rng);
        if drops.is_empty() {
            return;
        }
        for stack in drops {
            self.spawn_at_location(position, 0.0, stack);
        }
        if let Some(Species::Creeper(creeper)) = self
            .entities
            .store
            .get_mut(killer)
            .and_then(|e| e.living_mut())
            .and_then(|l| l.mob.as_mut())
            .map(|m| &mut m.species)
        {
            creeper.dropped_skulls = true;
        }
    }

    pub fn ground_item_count(&self) -> usize {
        self.entities
            .store
            .iter()
            .filter(|e| item::stack(&e.base).is_some())
            .count()
    }
}

fn merge_stack(stacks: &mut Vec<(BlockPos, ItemStack)>, pos: BlockPos, stack: ItemStack) {
    let mut remaining = u32::from(stack.count.get());
    for (_, existing) in stacks.iter_mut() {
        if remaining == 0 {
            return;
        }
        if !same_item_same_components(existing, &stack) {
            continue;
        }
        let room = MAX_MERGED_STACK.saturating_sub(u32::from(existing.count.get()));
        let moved = room.min(remaining);
        if moved > 0 {
            existing.count =
                std::num::NonZeroU8::new((u32::from(existing.count.get()) + moved) as u8)
                    .unwrap_or(existing.count);
            remaining -= moved;
        }
    }
    if let Some(count) = std::num::NonZeroU8::new(remaining as u8) {
        stacks.push((pos, ItemStack { count, ..stack }));
    }
}
