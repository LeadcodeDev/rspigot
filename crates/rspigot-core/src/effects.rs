use rspigot_domain::effect::{
    ApplyOutcome, AttributeState, EffectId, EffectInstance, affected_attribute, effect_attributes,
    player_base,
};
use rspigot_domain::player::PlayerId;

use crate::events::ClientEvent;
use crate::ports::ClientLink;
use crate::server::{Audience, Server};

const FLAG_EFFECTS: [&str; 2] = ["minecraft:invisibility", "minecraft:glowing"];

fn changes_flags(effects: &[EffectId]) -> bool {
    effects
        .iter()
        .any(|e| e.name().is_some_and(|name| FLAG_EFFECTS.contains(&name)))
}

impl<L: ClientLink> Server<L> {
    pub(crate) fn apply_effect(&mut self, id: PlayerId, instance: EffectInstance) -> bool {
        let Some(player) = self.players.get_mut(&id) else {
            return false;
        };
        let outcome = player.effects.apply(instance);
        if outcome == ApplyOutcome::Unchanged {
            return false;
        }
        if let Some(current) = player.effects.get(instance.effect).copied() {
            player.link.send(ClientEvent::EffectApplied {
                entity: player.entity_id,
                effect: current,
                blend: outcome == ApplyOutcome::Added,
            });
        }
        self.sync_effect_state(id, &[instance.effect]);
        true
    }

    pub(crate) fn remove_effects(&mut self, id: PlayerId, effect: Option<EffectId>) -> bool {
        let Some(player) = self.players.get_mut(&id) else {
            return false;
        };
        let removed: Vec<EffectId> = match effect {
            Some(effect) => player
                .effects
                .remove(effect)
                .map(|e| e.effect)
                .into_iter()
                .collect(),
            None => player
                .effects
                .clear()
                .into_iter()
                .map(|e| e.effect)
                .collect(),
        };
        if removed.is_empty() {
            return false;
        }
        for effect in &removed {
            player.link.send(ClientEvent::EffectRemoved {
                entity: player.entity_id,
                effect: *effect,
            });
        }
        self.sync_effect_state(id, &removed);
        true
    }

    pub(crate) fn tick_effects(&mut self) {
        let mut expired = Vec::new();
        for player in self.players.values_mut() {
            let ended = player.effects.tick();
            if ended.is_empty() {
                continue;
            }
            for effect in &ended {
                player.link.send(ClientEvent::EffectRemoved {
                    entity: player.entity_id,
                    effect: *effect,
                });
            }
            expired.push((player.id, ended));
        }
        for (id, ended) in expired {
            self.sync_effect_state(id, &ended);
        }
    }

    fn sync_effect_state(&mut self, id: PlayerId, changed: &[EffectId]) {
        let Some(player) = self.players.get(&id) else {
            return;
        };
        let entity = player.entity_id;
        let mut attributes = effect_attributes(&player.effects);
        for attribute in changed.iter().filter_map(|e| affected_attribute(*e)) {
            if !attributes.iter().any(|a| a.attribute == attribute) {
                attributes.push(AttributeState {
                    attribute,
                    base: player_base(attribute),
                    modifiers: Vec::new(),
                });
            }
        }
        attributes.retain(|a| {
            changed
                .iter()
                .filter_map(|e| affected_attribute(*e))
                .any(|c| c == a.attribute)
        });
        if !attributes.is_empty() {
            player.link.send(ClientEvent::Attributes {
                entity,
                attributes: attributes.clone(),
            });
            self.push(
                Audience::Trackers(entity),
                ClientEvent::Attributes { entity, attributes },
            );
        }
        if changes_flags(changed) {
            self.broadcast_metadata(id);
            if let Some(player) = self.players.get(&id) {
                player.link.send(ClientEvent::EntityMetadata {
                    entity,
                    entries: Self::metadata(player),
                });
            }
        }
    }
}
