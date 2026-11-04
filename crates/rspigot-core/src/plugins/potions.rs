use rspigot_domain::EntityId;
use rspigot_domain::effect::{ApplyOutcome, EffectDuration, EffectId, EffectInstance};
use rspigot_domain::player::PlayerId;

use crate::events::ClientEvent;
use crate::plugins::EventPayload;
use crate::plugins::decisions::Continuation;
use crate::plugins::events::{
    EntityPotionEffect, EntityPotionEffectAction, EntityPotionEffectCause, EntityPotionEffectPatch,
};
use crate::plugins::server::effect_spec;
use crate::plugins::steps::{Holder, Potion, PotionStep};
use crate::ports::ClientLink;
use crate::server::Server;

impl Potion {
    pub(crate) const fn instance(self) -> EffectInstance {
        match self {
            Self::Add(effect)
            | Self::Remove(effect)
            | Self::Clear(effect)
            | Self::Expire(effect) => effect,
        }
    }
}

impl<L: ClientLink> Server<L> {
    fn holder_entity(&self, holder: Holder) -> Option<EntityId> {
        match holder {
            Holder::Player(id) => self.players.get(&id).map(|player| player.entity_id),
            Holder::Mob(id) => Some(id),
        }
    }

    fn active_effect(&self, holder: Holder, effect: EffectId) -> Option<EffectInstance> {
        match holder {
            Holder::Player(id) => self.players.get(&id)?.effects.get(effect).copied(),
            Holder::Mob(id) => self
                .entities
                .store
                .get(id)?
                .living()?
                .effects
                .get(effect)
                .copied(),
        }
    }

    fn trial_effect(&self, holder: Holder, instance: EffectInstance) -> ApplyOutcome {
        let effects = match holder {
            Holder::Player(id) => self.players.get(&id).map(|player| &player.effects),
            Holder::Mob(id) => self
                .entities
                .store
                .get(id)
                .and_then(|entity| entity.living())
                .map(|living| &living.effects),
        };
        effects.map_or(ApplyOutcome::Unchanged, |effects| effects.trial(instance))
    }

    pub(crate) fn potion(
        &mut self,
        holder: Holder,
        potion: Potion,
        cause: EntityPotionEffectCause,
    ) -> bool {
        let Some(entity) = self.holder_entity(holder) else {
            return false;
        };
        let instance = potion.instance();
        let old = self.active_effect(holder, instance.effect);
        let (action, old, new, overrides) = match potion {
            Potion::Add(new) => {
                let outcome = self.trial_effect(holder, new);
                let action = if old.is_some() {
                    EntityPotionEffectAction::Changed
                } else {
                    EntityPotionEffectAction::Added
                };
                (action, old, Some(new), outcome != ApplyOutcome::Unchanged)
            }
            Potion::Remove(old) | Potion::Expire(old) => {
                (EntityPotionEffectAction::Removed, Some(old), None, true)
            }
            Potion::Clear(old) => (EntityPotionEffectAction::Cleared, Some(old), None, true),
        };
        let lane = self.lane_for(None, entity);
        let step = PotionStep {
            holder,
            potion,
            overrides,
            respawned: false,
        };
        self.fire(lane, Continuation::Potion(step), move |server| {
            let entity = server.subject_view(entity)?;
            Some(EventPayload::EntityPotionEffect(EntityPotionEffect {
                action,
                cause,
                entity_type: entity.kind.clone(),
                entity,
                modified_type: instance.effect.name()?.to_owned(),
                new_effect: new.as_ref().and_then(effect_spec),
                old_effect: old.as_ref().and_then(effect_spec),
                overrides,
            }))
        });
        overrides
    }

    pub(crate) fn clear_at_respawn(&mut self, id: PlayerId, effects: Vec<EffectInstance>) {
        let Some(entity) = self.players.get(&id).map(|player| player.entity_id) else {
            return;
        };
        let lane = self.lane_for(None, entity);
        for effect in effects {
            let step = PotionStep {
                holder: Holder::Player(id),
                potion: Potion::Clear(effect),
                overrides: true,
                respawned: true,
            };
            self.fire(lane, Continuation::Potion(step), move |server| {
                let entity = server.subject_view(entity)?;
                Some(EventPayload::EntityPotionEffect(EntityPotionEffect {
                    action: EntityPotionEffectAction::Cleared,
                    cause: EntityPotionEffectCause::Death,
                    entity_type: entity.kind.clone(),
                    entity,
                    modified_type: effect.effect.name()?.to_owned(),
                    new_effect: None,
                    old_effect: effect_spec(&effect),
                    overrides: true,
                }))
            });
        }
    }

    pub(super) fn finish_potion(
        &mut self,
        step: PotionStep,
        cancelled: bool,
        patch: EntityPotionEffectPatch,
    ) {
        let PotionStep {
            holder,
            potion,
            overrides,
            respawned,
        } = step;
        if cancelled {
            if respawned
                && let Holder::Player(id) = holder
                && let Some(player) = self.players.get(&id)
                && let Some(effect) = player.effects.get(potion.instance().effect).copied()
            {
                player.link.send(ClientEvent::EffectApplied {
                    entity: player.entity_id,
                    effect,
                    blend: false,
                });
            }
            return;
        }
        match potion {
            Potion::Add(new) => {
                let changed = self.active_effect(holder, new.effect).is_some();
                if changed && !patch.overrides.unwrap_or(overrides) {
                    if let Holder::Player(id) = holder {
                        self.start_effect(id, new);
                    }
                    return;
                }
                match holder {
                    Holder::Player(id) => {
                        self.apply_effect_now(id, new);
                    }
                    Holder::Mob(id) => {
                        self.apply_entity_effect_now(id, new);
                    }
                }
            }
            Potion::Remove(old) | Potion::Clear(old) | Potion::Expire(old) => {
                let current = self.active_effect(holder, old.effect);
                let spent = matches!(potion, Potion::Expire(_));
                let unchanged = current.is_some_and(|current| {
                    current.amplifier == old.amplifier
                        && (!spent || current.duration == EffectDuration::Ticks(0))
                });
                if !unchanged {
                    return;
                }
                match holder {
                    Holder::Player(id) => {
                        self.remove_effect_now(id, old.effect);
                    }
                    Holder::Mob(id) => {
                        self.remove_entity_effect_now(id, old.effect);
                    }
                }
            }
        }
    }
}
