use crate::plugins::events::{PluginDisable, PluginEnable, ServerLoad, ServerLoadType};
use crate::plugins::{EventPayload, PluginInfo};
use crate::ports::ClientLink;
use crate::server::Server;
use crate::status::PingHook;

impl<L: ClientLink> Server<L> {
    pub(crate) fn server_loaded(&mut self) {
        self.notify_now(EventPayload::ServerLoad(ServerLoad {
            load_type: ServerLoadType::Startup,
        }));
    }

    pub(crate) fn plugin_enabled(&mut self, connection: u64, plugin: PluginInfo) {
        self.notify_others(
            connection,
            &EventPayload::PluginEnable(PluginEnable { plugin }),
        );
    }

    pub(crate) fn plugin_disabled(&mut self, connection: u64, plugin: PluginInfo) {
        self.notify_others(
            connection,
            &EventPayload::PluginDisable(PluginDisable { plugin }),
        );
    }

    pub(crate) fn disable_plugins_at_shutdown(&mut self) {
        for (connection, plugin) in self.plugins.connected().into_iter().rev() {
            self.plugin_disabled(connection, plugin);
        }
        self.status.update(|status| status.ping = PingHook::Off);
    }
}
