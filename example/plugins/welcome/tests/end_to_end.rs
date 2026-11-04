use std::net::SocketAddr;
use std::process::{Child, Command};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use crossbeam_channel::{Receiver, RecvTimeoutError};
use rspigot_core::config::{GameConfig, LevelData};
use rspigot_core::events::{
    ClickInput, ClientEvent, ContainerClick, EntityKind, HashedSlot, Inbound, PlayerInput,
};
use rspigot_core::ports::{ChunkGenerator, ChunkStore, ClientLink, EntityStore, StoreError};
use rspigot_core::server::Server;
use rspigot_core::status::StatusBoard;
use rspigot_domain::chunk::ChunkColumn;
use rspigot_domain::player::{ClientSettings, GameProfile, PlayerId};
use rspigot_domain::text::Component;
use rspigot_domain::world::Dimension;
use rspigot_domain::{BiomeId, BlockPos, BlockStateId, ChunkPos, Identifier, Vec3};
use rspigot_entities::EntityKind as Kind;
use rspigot_items::ItemKind;
use rspigot_nbt::Compound;
use tokio::net::TcpListener;
use tokio::sync::watch;

const TICK: Duration = Duration::from_millis(50);
const PATIENCE: Duration = Duration::from_secs(10);
const SPAWN: Vec3 = Vec3::new(0.5, 64.0, 0.5);
const EAST: Vec3 = Vec3::new(1.5, 64.0, 0.5);
const LEAVE_SLOT: i16 = 4;

#[derive(Clone, Copy)]
struct Screen {
    window: i32,
    state: i32,
}

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
        SPAWN
    }
}

struct Player {
    id: PlayerId,
    link: Link,
    log: Vec<ClientEvent>,
}

impl Player {
    fn heard(&mut self) -> &[ClientEvent] {
        self.log.extend(self.link.take());
        &self.log
    }

    fn was_told(&mut self, text: &str) -> bool {
        self.heard().iter().any(|event| {
            matches!(event, ClientEvent::SystemChat { message, .. } if message.plain_text() == text)
        })
    }

    fn was_sent_to(&mut self, position: Vec3) -> bool {
        self.heard().iter().any(
            |event| matches!(event, ClientEvent::Teleport { position: to, .. } if *to == position),
        )
    }

    fn received_chunk(&mut self, pos: ChunkPos) -> bool {
        self.heard()
            .iter()
            .any(|event| matches!(event, ClientEvent::Chunk { chunk, .. } if chunk.pos() == pos))
    }

    fn saw_title(&mut self, text: &str) -> bool {
        self.heard().iter().any(
            |event| matches!(event, ClientEvent::TitleText(title) if title.plain_text() == text),
        )
    }

    fn was_given(&mut self, item: ItemKind, name: &str) -> bool {
        self.heard().iter().any(|event| {
            matches!(
                event,
                ClientEvent::InventorySlot { stack: Some(stack), .. }
                    if stack.item == item.id()
                        && stack.custom_name().is_some_and(|shown| shown.plain_text() == name)
            )
        })
    }

    fn saw_spawn_of(&mut self, kind: Kind) -> bool {
        let kind = EntityKind::Other(kind.id().0);
        self.heard().iter().any(
            |event| matches!(event, ClientEvent::SpawnEntity { kind: spawned, .. } if *spawned == kind),
        )
    }

    fn screen(&mut self, title: &str) -> Option<Screen> {
        let window = self.heard().iter().find_map(|event| match event {
            ClientEvent::OpenScreen {
                window,
                title: shown,
                ..
            } if shown.plain_text() == title => Some(*window),
            _ => None,
        })?;
        self.log.iter().rev().find_map(|event| match event {
            ClientEvent::ContainerContent {
                window: shown,
                state_id,
                ..
            } if *shown == window => Some(Screen {
                window,
                state: *state_id,
            }),
            _ => None,
        })
    }

    fn saw_closed(&mut self, screen: Screen) -> bool {
        self.heard()
            .contains(&ClientEvent::ContainerClose(screen.window))
    }
}

fn server() -> Server<Link> {
    let config = GameConfig {
        view_distance: 3,
        chunk_workers: 2,
        spawn_protection: 0,
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
        address: std::net::Ipv4Addr::LOCALHOST.into(),
        session: 0,
    });
    server.deliver();
    server.tick();
    let mut player = Player {
        id,
        link,
        log: Vec::new(),
    };
    assert!(
        player.received_chunk(SPAWN.chunk()),
        "{name} receives the spawn chunk before loading in"
    );
    confirm_teleports(server, &player);
    server.handle(Inbound::Input {
        player: id,
        input: PlayerInput::Loaded,
    });
    player
}

fn confirm_teleports(server: &mut Server<Link>, player: &Player) {
    for event in &player.log {
        if let ClientEvent::Teleport { id: teleport, .. } = event {
            server.handle(Inbound::Input {
                player: player.id,
                input: PlayerInput::TeleportConfirm(*teleport),
            });
        }
    }
}

fn step(server: &mut Server<Link>, player: &Player, to: Vec3) {
    server.handle(Inbound::Input {
        player: player.id,
        input: PlayerInput::Move {
            position: Some(to),
            rotation: None,
            on_ground: true,
        },
    });
    server.deliver();
}

fn click(server: &mut Server<Link>, player: &Player, screen: Screen, slot: i16) {
    server.handle(Inbound::Input {
        player: player.id,
        input: PlayerInput::ContainerClick(ContainerClick {
            container: screen.window,
            state_id: screen.state,
            slot,
            button: 0,
            input: ClickInput::Pickup,
            changed: Vec::new(),
            carried: HashedSlot::EMPTY,
        }),
    });
    server.deliver();
}

struct PluginProcess(Child);

impl PluginProcess {
    fn start(address: SocketAddr) -> Self {
        let child = Command::new(env!("CARGO_BIN_EXE_welcome"))
            .env("RSPIGOT_URL", format!("http://{address}"))
            .env_remove("RSPIGOT_TOKEN")
            .spawn()
            .expect("the welcome plugin starts");
        Self(child)
    }

    fn kill(&mut self) {
        let _ = self.0.kill();
        let _ = self.0.wait();
    }
}

impl Drop for PluginProcess {
    fn drop(&mut self) {
        self.kill();
    }
}

struct Core {
    server: Server<Link>,
    inbox: Receiver<Inbound<Link>>,
    welcome: Option<u64>,
    next_tick: Instant,
}

impl Core {
    fn new(inbox: Receiver<Inbound<Link>>) -> Self {
        Self {
            server: server(),
            inbox,
            welcome: None,
            next_tick: Instant::now(),
        }
    }

    fn run_until(&mut self, what: &str, mut done: impl FnMut(&Self) -> bool) {
        let given_up = Instant::now() + PATIENCE;
        while !done(self) {
            let now = Instant::now();
            assert!(now < given_up, "gave up waiting until {what}");
            if now >= self.next_tick {
                let inbox = self.inbox.clone();
                let waiting = inbox.len();
                for message in inbox.try_iter().take(waiting) {
                    self.handle(message);
                }
                self.server.tick();
                self.next_tick += TICK;
                continue;
            }
            match self.inbox.recv_timeout(self.next_tick - now) {
                Ok(message) => self.handle(message),
                Err(RecvTimeoutError::Timeout) => {}
                Err(RecvTimeoutError::Disconnected) => panic!("the plugin host stopped"),
            }
        }
    }

    fn handle(&mut self, message: Inbound<Link>) {
        match &message {
            Inbound::PluginConnected {
                connection, intro, ..
            } if intro.name == "welcome" => self.welcome = Some(*connection),
            Inbound::PluginDisconnected { connection } if self.welcome == Some(*connection) => {
                self.welcome = None;
            }
            _ => {}
        }
        self.server.handle(message);
        self.server.deliver();
    }
}

#[test]
fn a_plugin_greets_a_player_shows_its_menu_and_keeps_them_in_place() {
    let runtime = tokio::runtime::Runtime::new().unwrap();
    let listener = runtime.block_on(TcpListener::bind("127.0.0.1:0")).unwrap();
    let address = listener.local_addr().unwrap();
    let (inbound, inbox) = crossbeam_channel::unbounded::<Inbound<Link>>();
    let (_stop, stopped) = watch::channel(false);
    runtime.spawn(rspigot_plugins::serve(listener, None, inbound, stopped));
    let mut core = Core::new(inbox);
    let mut plugin = PluginProcess::start(address);

    core.run_until("the welcome plugin connects", |core| core.welcome.is_some());

    let mut alice = join(&mut core.server, "Alice");
    core.run_until("Alice is welcomed", |_| alice.was_told("Welcome Alice!"));
    core.run_until("Alice sees the welcome title", |_| {
        alice.saw_title("Welcome")
    });
    core.run_until("Alice gets a compass named Menu", |_| {
        alice.was_given(ItemKind::Compass, "Menu")
    });
    core.run_until("a chicken spawns at Alice's feet", |_| {
        alice.saw_spawn_of(Kind::Chicken)
    });
    let mut menu = None;
    core.run_until("Alice is shown the menu", |_| {
        menu = alice.screen("Menu");
        menu.is_some()
    });
    let menu = menu.expect("the menu is open");

    click(&mut core.server, &alice, menu, LEAVE_SLOT);
    core.run_until("the menu closes", |_| alice.saw_closed(menu));
    core.run_until("Alice is told goodbye", |_| {
        alice.was_told("See you around")
    });

    alice.log.clear();
    step(&mut core.server, &alice, EAST);
    let mut moved = false;
    core.run_until("Alice is sent back to the spawn", |core| {
        moved |= core.server.player_location(alice.id) == Some(EAST);
        alice.was_sent_to(SPAWN)
    });
    assert!(!moved, "the move was applied, then rolled back");
    assert_eq!(core.server.player_location(alice.id), Some(SPAWN));
    confirm_teleports(&mut core.server, &alice);

    plugin.kill();
    core.run_until("the welcome plugin disconnects", |core| {
        core.welcome.is_none()
    });

    alice.log.clear();
    step(&mut core.server, &alice, EAST);
    assert_eq!(core.server.player_location(alice.id), Some(EAST));
    assert!(!alice.was_sent_to(SPAWN));
}
