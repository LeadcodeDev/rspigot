use rspigot_domain::player::PlayerId;
use rspigot_domain::text::Component;

use crate::events::ClientEvent;
use crate::player::Player;
use crate::plugins::{
    Access, Answer, EventPayload, FromPlugin, HandlerSpec, Phase, PlayerSnapshot, PlayerState,
    ToPlugin,
};
use crate::ports::ClientLink;
use crate::server::{Audience, Server};

impl<L: ClientLink> Server<L> {
    pub(crate) fn next_event_id(&mut self) -> u64 {
        self.plugins.next_event_id()
    }

    pub(crate) fn notify_plugins(
        &mut self,
        id: u64,
        payload: &EventPayload,
        cancelled: bool,
        phase: Phase,
    ) {
        let listeners =
            self.plugins
                .listeners(payload.kind(), Access::Observe, &self.config.plugins);
        for (connection, handlers) in recipients(listeners, cancelled, phase) {
            self.plugins.send(
                connection,
                ToPlugin::Event {
                    id,
                    handlers,
                    phase,
                    cancelled,
                    payload: payload.clone(),
                },
            );
        }
    }

    pub(crate) fn snapshot_of(&self, id: PlayerId) -> Option<PlayerSnapshot> {
        self.players.get(&id).map(|player| self.describe(player))
    }

    pub(crate) fn plugin_message(&mut self, connection: u64, message: FromPlugin) {
        if !self.plugins.is_current(connection) {
            return;
        }
        match message {
            FromPlugin::Decision { .. } => {}
            FromPlugin::SendMessage { player, text } => self.push(
                Audience::One(player),
                ClientEvent::system(Component::text(text)),
            ),
            FromPlugin::ResolvePlayer { request, player } => {
                let answer = self
                    .state_of(player)
                    .map_or(Answer::NotFound, Answer::Player);
                self.plugins
                    .send(connection, ToPlugin::Answer { request, answer });
            }
            FromPlugin::FetchPlayers { request } => {
                let answer = Answer::Players(self.online_snapshots());
                self.plugins
                    .send(connection, ToPlugin::Answer { request, answer });
            }
        }
    }

    pub(crate) fn tell_plugins_of_join(&mut self, id: PlayerId) {
        self.settle_for_plugins(id, |player| EventPayload::PlayerJoin { player });
    }

    pub(crate) fn tell_plugins_of_leave(&mut self, id: PlayerId) {
        self.settle_for_plugins(id, |player| EventPayload::PlayerLeave { player });
    }

    fn settle_for_plugins(
        &mut self,
        id: PlayerId,
        payload: impl FnOnce(PlayerSnapshot) -> EventPayload,
    ) {
        if let Some(player) = self.snapshot_of(id) {
            let event = self.next_event_id();
            self.notify_plugins(event, &payload(player), false, Phase::Settled);
        }
    }

    fn describe(&self, player: &Player<L>) -> PlayerSnapshot {
        PlayerSnapshot {
            id: player.id,
            name: player.profile.name.clone(),
            entity_id: player.entity_id.0,
            world: self.world.dimension.key.to_string(),
            position: player.position,
            game_mode: player.game_mode,
        }
    }

    fn state_of(&self, id: PlayerId) -> Option<PlayerState> {
        self.players.get(&id).map(|player| PlayerState {
            player: self.describe(player),
            health: player.vitals.health,
            food: player.vitals.food.food,
            level: player.vitals.experience.level,
        })
    }

    fn online_snapshots(&self) -> Vec<PlayerSnapshot> {
        let mut online: Vec<PlayerSnapshot> = self
            .players
            .values()
            .map(|player| self.describe(player))
            .collect();
        online.sort_by_key(|snapshot| snapshot.entity_id);
        online
    }
}

fn recipients(
    listeners: Vec<(u64, HandlerSpec)>,
    cancelled: bool,
    phase: Phase,
) -> Vec<(u64, Vec<u32>)> {
    let skip_filtered = cancelled && phase == Phase::Settled;
    let mut grouped: Vec<(u64, Vec<u32>)> = Vec::new();
    for (connection, spec) in listeners {
        if skip_filtered && spec.ignore_cancelled {
            continue;
        }
        match grouped.iter_mut().find(|(known, _)| *known == connection) {
            Some((_, handlers)) => handlers.push(spec.id),
            None => grouped.push((connection, vec![spec.id])),
        }
    }
    grouped
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::plugins::{EventKind, Priority};

    fn listener(connection: u64, id: u32, ignore_cancelled: bool) -> (u64, HandlerSpec) {
        (
            connection,
            HandlerSpec {
                id,
                event: EventKind::PlayerMove,
                access: Access::Observe,
                priority: Priority::Normal,
                ignore_cancelled,
            },
        )
    }

    fn mixed() -> Vec<(u64, HandlerSpec)> {
        vec![
            listener(2, 4, false),
            listener(1, 1, true),
            listener(2, 5, true),
            listener(1, 2, false),
        ]
    }

    #[test]
    fn handlers_are_grouped_by_connection_in_the_order_they_were_listed() {
        assert_eq!(
            recipients(mixed(), false, Phase::Settled),
            [(2, vec![4, 5]), (1, vec![1, 2])]
        );
    }

    #[test]
    fn a_cancelled_event_skips_the_handlers_that_ignore_cancelled() {
        assert_eq!(
            recipients(mixed(), true, Phase::Settled),
            [(2, vec![4]), (1, vec![2])]
        );
    }

    #[test]
    fn a_rolled_back_event_still_reaches_the_handlers_that_ignore_cancelled() {
        assert_eq!(
            recipients(mixed(), true, Phase::RolledBack),
            [(2, vec![4, 5]), (1, vec![1, 2])]
        );
    }

    #[test]
    fn a_connection_whose_handlers_are_all_skipped_hears_nothing() {
        let only_filtered = vec![listener(1, 1, true), listener(3, 2, false)];
        assert_eq!(
            recipients(only_filtered, true, Phase::Settled),
            [(3, vec![2])]
        );
        assert!(recipients(Vec::new(), false, Phase::Settled).is_empty());
    }
}
