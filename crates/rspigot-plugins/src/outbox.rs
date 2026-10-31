use rspigot_core::plugins::ToPlugin;
use rspigot_core::ports::PluginOutbox;
use tokio::sync::mpsc::UnboundedSender;

pub(crate) struct Outbox(UnboundedSender<ToPlugin>);

impl Outbox {
    pub(crate) fn new(sender: UnboundedSender<ToPlugin>) -> Self {
        Self(sender)
    }
}

impl PluginOutbox for Outbox {
    fn send(&self, message: ToPlugin) {
        let _ = self.0.send(message);
    }
}
