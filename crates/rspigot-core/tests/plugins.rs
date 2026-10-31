use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use rspigot_core::config::{GameConfig, LevelData, PluginSettings};
use rspigot_core::events::{ClientEvent, Inbound, PlayerInput};
use rspigot_core::plugins::{PluginIntro, ToPlugin};
use rspigot_core::ports::{
    ChunkGenerator, ChunkStore, ClientLink, EntityStore, PluginOutbox, StoreError,
};
use rspigot_core::server::Server;
use rspigot_core::status::StatusBoard;
use rspigot_domain::chunk::ChunkColumn;
use rspigot_domain::player::{ClientSettings, GameProfile, PlayerId};
use rspigot_domain::text::Component;
use rspigot_domain::world::Dimension;
use rspigot_domain::{BiomeId, BlockPos, BlockStateId, ChunkPos, Identifier, Vec3};
use rspigot_nbt::Compound;

#[derive(Clone, Default)]
struct Link {
    events: Arc<Mutex<Vec<ClientEvent>>>,
    closed: Arc<AtomicBool>,
}

impl Link {
    fn take(&self) -> Vec<ClientEvent> {
        self.events
            .lock()
            .map(|mut e| std::mem::take(&mut *e))
            .unwrap_or_default()
    }
}

impl ClientLink for Link {
    fn send(&self, event: ClientEvent) {
        if let Ok(mut events) = self.events.lock() {
            events.push(event);
        }
    }

    fn disconnect(&self, _: Component) {
        self.closed.store(true, Ordering::Release);
    }

    fn is_connected(&self) -> bool {
        !self.closed.load(Ordering::Acquire)
    }
}

struct NoStore;

impl ChunkStore for NoStore {
    fn load(&self, _: &Identifier, _: ChunkPos) -> Result<Option<ChunkColumn>, StoreError> {
        Ok(None)
    }

    fn save(&self, _: &Identifier, _: &ChunkColumn) -> Result<(), StoreError> {
        Ok(())
    }

    fn flush(&self) -> Result<(), StoreError> {
        Ok(())
    }
}

impl EntityStore for NoStore {
    fn load_entities(
        &self,
        _: &Identifier,
        _: ChunkPos,
    ) -> Result<Option<Vec<Compound>>, StoreError> {
        Ok(None)
    }

    fn save_entities(&self, _: &Identifier, _: ChunkPos, _: &[Compound]) -> Result<(), StoreError> {
        Ok(())
    }
}

struct Floor;

impl ChunkGenerator for Floor {
    fn generate(&self, dimension: &Dimension, pos: ChunkPos) -> ChunkColumn {
        let mut chunk = ChunkColumn::empty(pos, dimension.height, BiomeId(0));
        for x in 0..16 {
            for z in 0..16 {
                for y in 60..=63 {
                    chunk.set_block(
                        BlockPos::new(pos.min_block_x() + x, y, pos.min_block_z() + z),
                        BlockStateId(1),
                    );
                }
            }
        }
        chunk
    }

    fn spawn_point(&self, _: &Dimension) -> Vec3 {
        Vec3::new(0.5, 64.0, 0.5)
    }
}

#[derive(Clone, Default)]
struct Mailbox(Arc<Mutex<Vec<ToPlugin>>>);

impl Mailbox {
    fn received(&self) -> Vec<ToPlugin> {
        self.0.lock().map(|m| m.clone()).unwrap_or_default()
    }
}

impl PluginOutbox for Mailbox {
    fn send(&self, message: ToPlugin) {
        if let Ok(mut inbox) = self.0.lock() {
            inbox.push(message);
        }
    }
}

struct Player {
    id: PlayerId,
    link: Link,
}

fn server(plugins: PluginSettings) -> Server<Link> {
    let config = GameConfig {
        view_distance: 3,
        chunk_workers: 2,
        spawn_protection: 0,
        plugins,
        ..GameConfig::default()
    };
    let mut server = Server::new(
        config,
        LevelData::default(),
        Arc::new(NoStore),
        Arc::new(Floor),
        StatusBoard::new(20),
    );
    server.wait_for_spawn(Duration::from_secs(10));
    server
}

fn join(server: &mut Server<Link>, name: &str) -> Player {
    let link = Link::default();
    let id = PlayerId::offline(name);
    server.handle(Inbound::Join {
        profile: GameProfile {
            id,
            name: name.to_owned(),
            properties: Vec::new(),
        },
        settings: ClientSettings::default(),
        link: link.clone(),
    });
    server.deliver();
    for event in link.take() {
        if let ClientEvent::Teleport { id: teleport, .. } = event {
            server.handle(Inbound::Input {
                player: id,
                input: PlayerInput::TeleportConfirm(teleport),
            });
        }
    }
    server.handle(Inbound::Input {
        player: id,
        input: PlayerInput::Loaded,
    });
    Player { id, link }
}

fn connect(server: &mut Server<Link>, connection: u64, name: &str) -> Mailbox {
    let mailbox = Mailbox::default();
    server.handle(Inbound::PluginConnected {
        connection,
        intro: PluginIntro {
            name: name.to_owned(),
            version: "1.0.0".to_owned(),
            handlers: Vec::new(),
        },
        outbox: Box::new(mailbox.clone()),
    });
    mailbox
}

fn plugins_line(server: &mut Server<Link>, player: &Player) -> String {
    server.handle(Inbound::Input {
        player: player.id,
        input: PlayerInput::Command("plugins".to_owned()),
    });
    server.deliver();
    player
        .link
        .take()
        .into_iter()
        .find_map(|event| match event {
            ClientEvent::SystemChat { message, .. } => Some(message.plain_text()),
            _ => None,
        })
        .expect("the plugins command answers")
}

#[test]
fn a_plugin_is_accepted_when_it_connects() {
    let mut server = server(PluginSettings::default());
    let welcome = connect(&mut server, 1, "welcome");
    assert_eq!(welcome.received(), [ToPlugin::Accepted]);
}

#[test]
fn the_plugins_command_lists_connected_plugins() {
    let mut server = server(PluginSettings::default());
    let alice = join(&mut server, "Alice");
    assert_eq!(plugins_line(&mut server, &alice), "Plugins (0): ");
    connect(&mut server, 1, "welcome");
    assert_eq!(plugins_line(&mut server, &alice), "Plugins (1): welcome");
}

#[test]
fn the_plugins_command_follows_the_configured_order() {
    let mut server = server(PluginSettings {
        order: vec!["guard".to_owned()],
        ..PluginSettings::default()
    });
    let alice = join(&mut server, "Alice");
    connect(&mut server, 1, "welcome");
    connect(&mut server, 2, "chat");
    connect(&mut server, 3, "guard");
    assert_eq!(
        plugins_line(&mut server, &alice),
        "Plugins (3): guard, chat, welcome"
    );
}

#[test]
fn a_plugin_that_takes_a_connected_name_replaces_the_old_connection() {
    let mut server = server(PluginSettings::default());
    let alice = join(&mut server, "Alice");
    let old = connect(&mut server, 1, "welcome");
    let new = connect(&mut server, 2, "welcome");
    assert_eq!(old.received(), [ToPlugin::Accepted, ToPlugin::Replaced]);
    assert_eq!(new.received(), [ToPlugin::Accepted]);
    server.handle(Inbound::PluginDisconnected { connection: 1 });
    assert_eq!(plugins_line(&mut server, &alice), "Plugins (1): welcome");
}

#[test]
fn a_disconnected_plugin_leaves_the_list() {
    let mut server = server(PluginSettings::default());
    let alice = join(&mut server, "Alice");
    connect(&mut server, 1, "welcome");
    server.handle(Inbound::PluginDisconnected { connection: 1 });
    assert_eq!(plugins_line(&mut server, &alice), "Plugins (0): ");
}
