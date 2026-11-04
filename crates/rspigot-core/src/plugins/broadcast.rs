use std::collections::VecDeque;

use rspigot_domain::player::PlayerId;

use crate::commands::CommandContext;
use crate::console::console_text;
use crate::events::ClientEvent;
use crate::plugins::decisions::{Continuation, Lane};
use crate::plugins::events::{BroadcastMessage, BroadcastMessagePatch};
use crate::plugins::resolve;
use crate::plugins::{EventKind, EventPayload, SenderView};
use crate::ports::ClientLink;
use crate::server::{Audience, Server};

const REACTION_TICKS: u64 = 6000;

#[derive(Debug, Default)]
pub(crate) struct Broadcasts(VecDeque<(u64, u64)>);

impl Broadcasts {
    fn remember(&mut self, event: u64, tick: u64) {
        self.forget_before(tick);
        self.0.push_back((event, tick));
    }

    fn heard(&mut self, event: u64, tick: u64) -> bool {
        self.forget_before(tick);
        self.0.binary_search_by_key(&event, |(id, _)| *id).is_ok()
    }

    fn forget_before(&mut self, tick: u64) {
        let oldest = tick.saturating_sub(REACTION_TICKS);
        while self.0.front().is_some_and(|(_, at)| *at < oldest) {
            self.0.pop_front();
        }
    }
}

impl<L: ClientLink> Server<L> {
    pub(super) fn broadcast_order(&mut self, origin: Option<u64>, message: String) {
        let tick = self.current_tick();
        let reaction = origin.is_some_and(|event| self.broadcasts.heard(event, tick));
        if reaction || !self.listened(EventKind::BroadcastMessage) {
            CommandContext::broadcast(self, resolve::text(&message));
            return;
        }
        let event = self.next_event_id();
        self.broadcasts.remember(event, tick);
        let text = message.clone();
        self.fire_numbered(
            Lane::Broadcast,
            event,
            Continuation::Broadcast(message),
            move |server| {
                let recipients = std::iter::once(SenderView::Console)
                    .chain(
                        server
                            .online_snapshots()
                            .into_iter()
                            .map(SenderView::Player),
                    )
                    .collect();
                Some(EventPayload::BroadcastMessage(BroadcastMessage {
                    message: text,
                    recipients,
                }))
            },
        );
    }

    pub(super) fn finish_broadcast(&mut self, message: &str, patch: BroadcastMessagePatch) {
        let line = resolve::text(patch.message.as_deref().unwrap_or(message));
        let Some(recipients) = patch.recipients else {
            CommandContext::broadcast(self, line);
            return;
        };
        let mut console = false;
        let mut players: Vec<PlayerId> = Vec::with_capacity(recipients.len());
        for recipient in recipients {
            match recipient {
                SenderView::Console => console = true,
                SenderView::Player(player) if !players.contains(&player.id) => {
                    players.push(player.id);
                }
                SenderView::Player(_) => {}
            }
        }
        if console {
            tracing::info!("{}", console_text(&line));
        }
        for player in players {
            self.push(Audience::One(player), ClientEvent::system(line.clone()));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_broadcast_event_is_remembered_for_the_reaction_window_only() {
        let mut broadcasts = Broadcasts::default();
        broadcasts.remember(4, 10);
        broadcasts.remember(9, 20);
        assert!(broadcasts.heard(4, 10 + REACTION_TICKS));
        assert!(broadcasts.heard(9, 10 + REACTION_TICKS));
        assert!(!broadcasts.heard(5, 10 + REACTION_TICKS));
        assert!(!broadcasts.heard(4, 11 + REACTION_TICKS));
        assert!(broadcasts.heard(9, 20 + REACTION_TICKS));
        assert!(!broadcasts.heard(9, 21 + REACTION_TICKS));
    }
}
