use crate::commands::{self, CommandSender};
use crate::events::ClientEvent;
use crate::plugins::decisions::{Continuation, Lane};
use crate::plugins::events::{ServerCommand, ServerCommandPatch, TabComplete, TabCompletePatch};
use crate::plugins::steps::SuggestStep;
use crate::plugins::{EventKind, EventPayload, SenderView};
use crate::ports::ClientLink;
use crate::server::{Audience, Server};

impl<L: ClientLink> Server<L> {
    pub(crate) fn console_line(&mut self, line: String) {
        if !self.listened(EventKind::ServerCommand) {
            self.finish_server_command(line, ServerCommandPatch::default());
            return;
        }
        let command = line.clone();
        self.fire(
            Lane::Console,
            Continuation::ServerCommand(line),
            move |_| {
                Some(EventPayload::ServerCommand(ServerCommand {
                    command,
                    sender: SenderView::Console,
                }))
            },
        );
    }

    pub(super) fn finish_server_command(&mut self, line: String, patch: ServerCommandPatch) {
        if !self.running {
            return;
        }
        let line = patch.command.unwrap_or(line);
        commands::execute(self, CommandSender::Console, &line);
    }

    pub(crate) fn ask_tab_complete(&mut self, buffer: &str, step: SuggestStep) {
        if !self.listened(EventKind::TabComplete) {
            self.finish_tab_complete(step, false, TabCompletePatch::default());
            return;
        }
        let player = step.player;
        let buffer = buffer.to_owned();
        let completions = step.matches.clone();
        self.fire(
            player,
            Continuation::TabComplete(Box::new(step)),
            move |server| {
                let sender = SenderView::Player(server.snapshot_of(player)?);
                Some(EventPayload::TabComplete(TabComplete {
                    buffer,
                    completions,
                    sender,
                }))
            },
        );
    }

    pub(super) fn finish_tab_complete(
        &mut self,
        step: SuggestStep,
        cancelled: bool,
        patch: TabCompletePatch,
    ) {
        if cancelled {
            return;
        }
        let matches = patch.completions.unwrap_or(step.matches);
        if matches.is_empty() {
            return;
        }
        self.push(
            Audience::One(step.player),
            ClientEvent::CommandSuggestions {
                id: step.request,
                start: step.start,
                length: step.length,
                matches,
            },
        );
    }
}
