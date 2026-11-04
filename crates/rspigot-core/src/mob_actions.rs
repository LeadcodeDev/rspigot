use rand::Rng;
use rspigot_ai::{Hand, MobAction, is_edible_for_sheep};
use rspigot_blocks::BlockKind;
use rspigot_domain::entity::ageable::{self, PARENT_AGE_AFTER_BREEDING, SHEEP_REGROWTH_SECONDS};
use rspigot_domain::entity::living::{self, LivingState};
use rspigot_domain::entity::species;
use rspigot_domain::entity::{Entity, EntityBase, EntityState};
use rspigot_domain::random::{LegacyRandom, RandomSource};
use rspigot_domain::{BlockPos, BlockStateId, EntityId, Vec3};
use rspigot_entities::{EntityKind as Kind, SoundSource};

use crate::commands::Target;
use crate::events::{Animation, ClientEvent, EntityStatus};
use crate::plugins::events::CreatureSpawnReason;
use crate::ports::ClientLink;
use crate::server::{Audience, Server};

pub const LOVE_EVENT: i8 = 18;
const BLOCK_BREAK_EVENT: i32 = 2001;
const BREED_EXPERIENCE: i32 = 7;
const TURTLE_EGG_BREAK: &str = "minecraft:entity.turtle.egg_break";

pub(crate) fn new_mob(base: EntityBase) -> Entity {
    let mut base = base;
    let living = LivingState::new(&mut base, 0.0);
    Entity::new(base, EntityState::Living(Box::new(living)))
}

impl<L: ClientLink> Server<L> {
    pub(crate) fn apply_mob_action(&mut self, id: EntityId, action: MobAction) {
        match action {
            MobAction::Swing(hand) => self.mob_swing(id, hand),
            MobAction::Attack(target) => {
                let target = match self.players.values().find(|p| p.entity_id == target) {
                    Some(player) => Target::Player(player.id),
                    None => Target::Entity(target),
                };
                self.do_hurt_target(id, target);
            }
            MobAction::RangedAttack { target, power } => {
                self.perform_ranged_attack(id, target, power)
            }
            MobAction::Breed(partner) => self.spawn_child_from_breeding(id, partner),
            MobAction::EntityEvent(event) => self.push(
                Audience::Trackers(id),
                ClientEvent::EntityStatus {
                    entity: id,
                    status: EntityStatus::Event(event),
                },
            ),
            MobAction::EatGrass => self.eat_grass(id),
            MobAction::Alert { mob, target } => {
                if let Some(entry) = self.entities.mobs.entries.get_mut(&mob)
                    && entry.ai.target().is_none()
                {
                    entry.ai.set_target(Some(target));
                }
            }
            MobAction::Particle(particle) => {
                self.push(Audience::Trackers(id), ClientEvent::Particle(particle));
            }
            MobAction::BlockSound {
                sound,
                source,
                pos,
                volume,
                pitch,
            } => {
                let at = Vec3::new(
                    f64::from(pos.x) + 0.5,
                    f64::from(pos.y) + 0.5,
                    f64::from(pos.z) + 0.5,
                );
                self.play_sound(None, at, sound, source, volume, pitch);
            }
            MobAction::RemoveTurtleEgg(pos) => {
                self.set_block(pos, BlockStateId::AIR);
                let pitch = 0.9 + self.rng.random::<f32>() * 0.2;
                let at = Vec3::new(
                    f64::from(pos.x) + 0.5,
                    f64::from(pos.y) + 0.5,
                    f64::from(pos.z) + 0.5,
                );
                self.play_sound(None, at, TURTLE_EGG_BREAK, SoundSource::Blocks, 0.7, pitch);
            }
            MobAction::LevelEvent { event, pos, data } => self.level_event(event, pos, data),
            MobAction::DestroyProgress { pos, stage } => self.push(
                Audience::ChunkViewers(pos.chunk()),
                ClientEvent::BlockDestruction {
                    entity: id,
                    pos,
                    stage: stage as i8,
                },
            ),
            MobAction::BreakDoor(pos) => {
                self.set_block(pos, BlockStateId::AIR);
                self.level_event(rspigot_ai::ZOMBIE_BREAK_DOOR_EVENT, pos, 0);
                let state = self.world.chunks.block(pos).unwrap_or(BlockStateId::AIR);
                self.level_event(BLOCK_BREAK_EVENT, pos, i32::from(state.0));
            }
        }
    }

    pub(crate) fn mob_swing(&mut self, id: EntityId, hand: Hand) {
        let started = self
            .entities
            .store
            .get_mut(id)
            .and_then(|e| e.living_mut())
            .is_some_and(living::start_swing);
        if started {
            let animation = match hand {
                Hand::Main => Animation::SwingMainArm,
                Hand::Off => Animation::SwingOffHand,
            };
            self.push(
                Audience::Trackers(id),
                ClientEvent::EntityAnimation {
                    entity: id,
                    animation,
                },
            );
        }
    }

    fn eat_grass(&mut self, id: EntityId) {
        let Some(pos) = self.entities.store.get(id).map(|e| e.base.position.block()) else {
            return;
        };
        let griefing = self.config.rules.mob_griefing;
        let here = self.world.chunks.block(pos);
        if is_edible_for_sheep(here) {
            if griefing {
                self.destroy_block_silently(pos);
            }
            self.mob_ate(id);
            return;
        }
        let below = pos.offset(0, -1, 0);
        if self.world.chunks.block(below).and_then(BlockStateId::kind)
            == Some(BlockKind::GrassBlock)
        {
            if griefing {
                self.level_event(
                    BLOCK_BREAK_EVENT,
                    below,
                    i32::from(BlockKind::GrassBlock.default_state().0),
                );
                self.set_block(below, BlockKind::Dirt.default_state());
            }
            self.mob_ate(id);
        }
    }

    pub(crate) fn destroy_block_silently(&mut self, pos: BlockPos) {
        let Some(state) = self.world.chunks.block(pos) else {
            return;
        };
        if state.is_air() {
            return;
        }
        self.level_event(BLOCK_BREAK_EVENT, pos, i32::from(state.0));
        self.set_block(pos, BlockStateId::AIR);
    }

    fn mob_ate(&mut self, id: EntityId) {
        let Some(Entity {
            base,
            state: EntityState::Living(state),
            ..
        }) = self.entities.store.get_mut(id)
        else {
            return;
        };
        if base.kind != Kind::Sheep {
            return;
        }
        species::set_sheared(base, false);
        if let Some(ageable) = state.mob.as_mut().and_then(|mob| mob.ageable.as_mut())
            && ageable::can_age_up(base, ageable)
        {
            ageable::age_up(base, ageable, SHEEP_REGROWTH_SECONDS, false);
        }
    }

    pub(crate) fn breed_offspring(
        &mut self,
        parent: EntityId,
        partner: EntityId,
        random: &mut LegacyRandom,
    ) -> Option<Entity> {
        let parent_entity = self.entities.store.get(parent)?;
        let kind = parent_entity.base.kind;
        if !matches!(kind, Kind::Cow | Kind::Pig | Kind::Chicken | Kind::Sheep) {
            return None;
        }
        let parent_variant = parent_entity
            .living()?
            .mob
            .as_ref()?
            .species
            .farm()
            .map(|farm| (farm, species::variant(&parent_entity.base, farm)));
        let parent_color = species::wool_color(&parent_entity.base);
        let partner_entity = self.entities.store.get(partner)?;
        let partner_variant = partner_entity
            .living()
            .and_then(|l| l.mob.as_ref())
            .and_then(|mob| mob.species.farm())
            .map(|farm| species::variant(&partner_entity.base, farm));
        let partner_color = species::wool_color(&partner_entity.base);
        let partner_kind = partner_entity.base.kind;
        let base = self.new_entity_base(kind);
        let mut child = new_mob(base);
        if let Some((farm, variant)) = parent_variant
            && partner_kind == kind
        {
            let chosen = if random.next_boolean() {
                variant
            } else {
                partner_variant.unwrap_or(variant)
            };
            species::set_variant(&mut child.base, farm, chosen);
        }
        if kind == Kind::Sheep {
            let mut level = crate::mobs::ServerRandom(&mut self.rng);
            let color = species::mixed_wool_color(parent_color, partner_color, &mut level);
            species::set_wool_color(&mut child.base, color);
        }
        Some(child)
    }

    pub(crate) fn spawn_child_from_breeding(&mut self, parent: EntityId, partner: EntityId) {
        let mut random = self.mob_random(parent);
        let Some(mut child) = self.breed_offspring(parent, partner, &mut random) else {
            self.store_mob_random(parent, random);
            return;
        };
        let Some(position) = self.entities.store.get(parent).map(|e| e.base.position) else {
            return;
        };
        if let EntityState::Living(state) = &mut child.state
            && let Some(ageable) = state.mob.as_mut().and_then(|mob| mob.ageable.as_mut())
        {
            ageable::set_baby(&mut child.base, ageable, true);
        }
        child.base.snap_to(position, 0.0, 0.0);
        let experience = random.next_int(BREED_EXPERIENCE) + 1;
        self.store_mob_random(parent, random);
        for id in [parent, partner] {
            if let Some(Entity {
                base,
                state: EntityState::Living(state),
                ..
            }) = self.entities.store.get_mut(id)
                && let Some(ageable) = state.mob.as_mut().and_then(|mob| mob.ageable.as_mut())
            {
                ageable::set_age(base, ageable, PARENT_AGE_AFTER_BREEDING);
                ageable.reset_love();
            }
        }
        self.push(
            Audience::Trackers(parent),
            ClientEvent::EntityStatus {
                entity: parent,
                status: EntityStatus::Event(LOVE_EVENT),
            },
        );
        if self.config.rules.mob_drops && experience > 0 {
            self.spawn_orb(position, experience);
        }
        self.add_entity(child, CreatureSpawnReason::Breeding);
    }

    pub(crate) fn set_in_love(&mut self, id: EntityId, cause: Option<uuid::Uuid>) {
        let Some(state) = self.entities.store.get_mut(id).and_then(|e| e.living_mut()) else {
            return;
        };
        let Some(ageable) = state.mob.as_mut().and_then(|mob| mob.ageable.as_mut()) else {
            return;
        };
        ageable.set_in_love(cause);
        self.push(
            Audience::Trackers(id),
            ClientEvent::EntityStatus {
                entity: id,
                status: EntityStatus::Event(LOVE_EVENT),
            },
        );
    }
}
