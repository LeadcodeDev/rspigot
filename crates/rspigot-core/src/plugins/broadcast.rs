use rspigot_domain::player::PlayerId;

use crate::commands::CommandContext;
use crate::console::console_text;
use crate::events::ClientEvent;
use crate::plugins::decisions::{Continuation, Lane};
use crate::plugins::events::{BroadcastMessage, BroadcastMessagePatch};
use crate::plugins::resolve;
use crate::plugins::{EventPayload, SenderView};
use crate::ports::ClientLink;
use crate::server::{Audience, Server};

impl<L: ClientLink> Server<L> {
    pub(super) fn broadcast_order(&mut self, connection: u64, message: String) {
        if self.decisions.owes(connection, Lane::Broadcast) {
            CommandContext::broadcast(self, resolve::text(&message));
            return;
        }
        let text = message.clone();
        self.fire(
            Lane::Broadcast,
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
