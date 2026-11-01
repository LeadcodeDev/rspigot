#![allow(dead_code)]

use std::net::IpAddr;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use rspigot_core::config::{GameConfig, LevelData, PluginSettings};
use rspigot_core::events::{ClientEvent, Inbound, PlayerInput};
use rspigot_core::plugins::{
    Access, Answer, EventKind, FromPlugin, HandlerSpec, Order, PlayerOrder, PluginIntro, Priority,
    Question, ToPlugin,
};
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

static NEXT_REQUEST: AtomicU64 = AtomicU64::new(1);

#[derive(Clone, Default)]
pub struct Link {
    events: Arc<Mutex<Vec<ClientEvent>>>,
    closed: Arc<AtomicBool>,
    reason: Arc<Mutex<Option<Component>>>,
}

impl Link {
    pub fn take(&self) -> Vec<ClientEvent> {
        self.events
            .lock()
            .map(|mut e| std::mem::take(&mut *e))
            .unwrap_or_default()
    }

    pub fn disconnected_with(&self) -> Option<Component> {
        self.reason.lock().ok().and_then(|reason| reason.clone())
    }
}

impl ClientLink for Link {
    fn send(&self, event: ClientEvent) {
        if let Ok(mut events) = self.events.lock() {
            events.push(event);
        }
    }

    fn disconnect(&self, reason: Component) {
        if let Ok(mut last) = self.reason.lock() {
            *last = Some(reason);
        }
        self.closed.store(true, Ordering::Release);
    }

    fn is_connected(&self) -> bool {
        !self.closed.load(Ordering::Acquire)
    }
}

pub struct NoStore;

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

pub struct Floor;

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
pub struct Mailbox(Arc<Mutex<Vec<ToPlugin>>>);

impl Mailbox {
    pub fn received(&self) -> Vec<ToPlugin> {
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

#[derive(Clone)]
pub struct Player {
    pub id: PlayerId,
    pub link: Link,
}

pub fn server(plugins: PluginSettings) -> Server<Link> {
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

pub fn join(server: &mut Server<Link>, name: &str) -> Player {
    join_from(server, name, IpAddr::from([127, 0, 0, 1]))
}

pub fn join_from(server: &mut Server<Link>, name: &str, address: IpAddr) -> Player {
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
        address,
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

pub fn observe(id: u32, event: EventKind) -> HandlerSpec {
    HandlerSpec {
        id,
        event,
        access: Access::Observe,
        priority: Priority::Normal,
        ignore_cancelled: false,
    }
}

pub fn decide(id: u32) -> HandlerSpec {
    HandlerSpec {
        id,
        event: EventKind::PlayerMove,
        access: Access::Decide,
        priority: Priority::Normal,
        ignore_cancelled: false,
    }
}

pub fn plug(
    server: &mut Server<Link>,
    connection: u64,
    name: &str,
    handlers: Vec<HandlerSpec>,
    outbox: Box<dyn PluginOutbox>,
) {
    server.handle(Inbound::PluginConnected {
        connection,
        intro: PluginIntro {
            name: name.to_owned(),
            version: "1.0.0".to_owned(),
            handlers,
        },
        outbox,
    });
}

pub fn connect_with(
    server: &mut Server<Link>,
    connection: u64,
    name: &str,
    handlers: Vec<HandlerSpec>,
) -> Mailbox {
    let mailbox = Mailbox::default();
    plug(
        server,
        connection,
        name,
        handlers,
        Box::new(mailbox.clone()),
    );
    mailbox
}

pub fn connect(server: &mut Server<Link>, connection: u64, name: &str) -> Mailbox {
    connect_with(server, connection, name, Vec::new())
}

pub fn heard(mailbox: &Mailbox) -> Vec<ToPlugin> {
    mailbox
        .received()
        .into_iter()
        .filter(|message| *message != ToPlugin::Accepted)
        .collect()
}

pub fn order(server: &mut Server<Link>, connection: u64, message: FromPlugin) {
    server.handle(Inbound::PluginMessage {
        connection,
        message,
    });
    server.deliver();
}

pub fn plugins_line(server: &mut Server<Link>, player: &Player) -> String {
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

pub fn texts(player: &Player) -> Vec<(String, bool)> {
    player
        .link
        .take()
        .into_iter()
        .filter_map(|event| match event {
            ClientEvent::SystemChat { message, overlay } => Some((message.plain_text(), overlay)),
            _ => None,
        })
        .collect()
}

pub struct Scene {
    pub server: Server<Link>,
    pub plugin: Mailbox,
    pub players: Vec<Player>,
}

pub fn scene(names: &[&str]) -> Scene {
    let mut server = server(PluginSettings::default());
    let plugin = connect(&mut server, 1, "tester");
    let players: Vec<Player> = names.iter().map(|name| join(&mut server, name)).collect();
    server.deliver();
    for player in &players {
        player.link.take();
    }
    Scene {
        server,
        plugin,
        players,
    }
}

impl Scene {
    pub fn act(&mut self, order: Order) {
        self.server.handle(Inbound::PluginMessage {
            connection: 1,
            message: FromPlugin::Order(order),
        });
        self.server.deliver();
    }

    pub fn ask(&mut self, question: Question) -> Answer {
        let request = NEXT_REQUEST.fetch_add(1, Ordering::Relaxed);
        self.server.handle(Inbound::PluginMessage {
            connection: 1,
            message: FromPlugin::Question { request, question },
        });
        self.server.deliver();
        self.plugin
            .received()
            .into_iter()
            .find_map(|message| match message {
                ToPlugin::Answer {
                    request: answered,
                    answer,
                } if answered == request => Some(answer),
                _ => None,
            })
            .expect("the server answers the question")
    }

    pub fn warnings(&self) -> Vec<String> {
        self.plugin
            .received()
            .into_iter()
            .filter_map(|message| match message {
                ToPlugin::Warning(text) => Some(text),
                _ => None,
            })
            .collect()
    }

    pub fn tick(&mut self, ticks: u32) {
        for _ in 0..ticks {
            self.server.tick();
            self.server.deliver();
        }
    }
}

pub fn stream_chunk_of(s: &mut Scene, pos: BlockPos) {
    let mut waiting = s.players.clone();
    for _ in 0..40 {
        if waiting.is_empty() {
            break;
        }
        s.tick(1);
        waiting.retain(|player| {
            !player.link.take().iter().any(
                |event| matches!(event, ClientEvent::Chunk { chunk, .. } if chunk.pos() == pos.chunk()),
            )
        });
    }
    assert!(waiting.is_empty(), "every client received the chunk");
}

pub fn to(player: &Player, order: PlayerOrder) -> Order {
    Order::Player {
        player: player.id,
        order,
    }
}
