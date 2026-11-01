use rand::Rng;
use rspigot_domain::damage::DamageType;
use rspigot_domain::entity::{Entity, RemovalReason, living};
use rspigot_domain::text::Component;
use rspigot_domain::{EntityId, Vec3};
use rspigot_entities::EntityKind as Kind;

use crate::commands::{CommandContext, Target};
use crate::console::console_text;
use crate::context::{SUMMON_DUPLICATE_UUID, SUMMON_FAILED};
use crate::health::DamageSource;
use crate::items::PICKUP_DELAY;
use crate::plugins::items::stack_of;
use crate::plugins::resolve::{self, finite_not_negative};
use crate::plugins::{Answer, EntityDetails, EntityOrder, EntityView, ItemSpec};
use crate::ports::ClientLink;
use crate::server::Server;

impl<L: ClientLink> Server<L> {
    pub(super) fn apply_entity_order(&mut self, connection: u64, id: EntityId, order: EntityOrder) {
        if self.entities.store.get(id).is_none() {
            return;
        }
        let name = order.name();
        if let Err(reason) = self.entity_order(id, order) {
            self.warn_plugin(connection, name, reason);
        }
    }

    fn entity_order(&mut self, id: EntityId, order: EntityOrder) -> Result<(), String> {
        let target = Target::Entity(id);
        match order {
            EntityOrder::Teleport(position) => {
                let position = resolve::position_of(position)?;
                CommandContext::teleport_target(self, target, position);
            }
            EntityOrder::Remove => {
                self.remove_entity(id, RemovalReason::Discarded);
            }
            EntityOrder::Damage(amount) => {
                let amount = resolve::damage_of(amount)?;
                self.hurt_entity(id, &DamageSource::new(DamageType::Generic), amount);
            }
            EntityOrder::Kill => {
                CommandContext::kill_target(self, target);
            }
            EntityOrder::AddEffect(spec) => {
                let effect = resolve::effect_of(&spec)?;
                CommandContext::give_effect_to(self, target, effect);
            }
            EntityOrder::RemoveEffect(key) => {
                let effect = resolve::effect_id(&key)?;
                CommandContext::clear_effect_of(self, target, Some(effect));
            }
            EntityOrder::ClearEffects => {
                CommandContext::clear_effect_of(self, target, None);
            }
            EntityOrder::SetVelocity(velocity) => {
                let velocity = resolve::velocity_of(velocity)?;
                if let Some(entity) = self.entities.store.get_mut(id) {
                    entity.base.velocity = velocity;
                    entity.base.needs_sync = true;
                }
            }
        }
        Ok(())
    }

    pub(crate) fn entity_view(&self, id: EntityId) -> Option<EntityView> {
        self.entities.store.get(id).map(view_of)
    }

    pub(super) fn spawned(&self, id: EntityId) -> Answer {
        self.entity_view(id)
            .map_or(Answer::NotFound, Answer::Spawned)
    }

    pub(super) fn entity_details(&self, id: EntityId) -> Option<EntityDetails> {
        let entity = self.entities.store.get(id)?;
        let base = &entity.base;
        Some(EntityDetails {
            entity: view_of(entity),
            health: entity.living().map(|_| living::health(base)),
            velocity: base.velocity,
            on_ground: base.on_ground,
            custom_name: base.custom_name().map(Component::to_legacy),
            fire_ticks: self.entity_fire_ticks(id)?,
        })
    }

    pub(super) fn listed_entities(
        &self,
        near: Option<(Vec3, f64)>,
        kind: Option<&str>,
    ) -> Result<Vec<EntityView>, String> {
        let near = match near {
            Some((center, radius)) => {
                finite_not_negative("radius", radius)?;
                Some((resolve::position_of(center)?, radius * radius))
            }
            None => None,
        };
        let kind = kind.map(entity_kind).transpose()?;
        let mut views: Vec<EntityView> = self
            .entities
            .store
            .iter()
            .filter(|entity| kind.is_none_or(|kind| entity.base.kind == kind))
            .filter(|entity| {
                near.is_none_or(|(center, reach)| {
                    entity.base.position.distance_squared(center) <= reach
                })
            })
            .map(view_of)
            .collect();
        views.sort_by_key(|view| view.id);
        Ok(views)
    }

    pub(super) fn spawn_for_plugin(
        &mut self,
        kind: &str,
        position: Vec3,
    ) -> Result<EntityId, String> {
        let kind = entity_kind(kind)?;
        let position = resolve::position_of(position)?;
        self.summon_entity(kind, position, None).map_err(refusal)
    }

    pub(super) fn drop_for_plugin(
        &mut self,
        position: Vec3,
        item: &ItemSpec,
    ) -> Result<EntityId, String> {
        let position = resolve::position_of(position)?;
        let stack = stack_of(item)?.ok_or_else(|| "air cannot be dropped".to_owned())?;
        if !self.world.chunks.is_loaded(position.chunk()) {
            return Err(refusal(SUMMON_FAILED));
        }
        let velocity = Vec3::new(
            self.rng.random::<f64>() * 0.2 - 0.1,
            0.2,
            self.rng.random::<f64>() * 0.2 - 0.1,
        );
        self.spawn_item_entity(position, velocity, PICKUP_DELAY, stack)
            .ok_or_else(|| refusal(SUMMON_DUPLICATE_UUID))
    }
}

fn view_of(entity: &Entity) -> EntityView {
    let base = &entity.base;
    EntityView {
        id: base.id,
        uuid: base.uuid,
        kind: base.kind.name().to_owned(),
        position: base.position,
    }
}

fn entity_kind(key: &str) -> Result<Kind, String> {
    Kind::from_name(key).ok_or_else(|| format!("unknown entity type {key}"))
}

fn refusal(key: &str) -> String {
    console_text(&Component::translatable(key, Vec::new()))
}
