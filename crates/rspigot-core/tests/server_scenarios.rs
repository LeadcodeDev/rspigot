use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use rspigot_core::admin::{AdminLists, ListEntry};
use rspigot_core::config::{GameConfig, LevelData};
use rspigot_core::events::{ClientEvent, DigAction, Direction, Inbound, PlayerInput};
use rspigot_core::ports::{ChunkGenerator, ChunkStore, ClientLink, StoreError};
use rspigot_core::server::Server;
use rspigot_core::status::StatusBoard;
use rspigot_domain::chunk::ChunkColumn;
use rspigot_domain::player::{ClientSettings, GameMode, GameProfile, PlayerId};
use rspigot_domain::text::Component;
use rspigot_domain::world::Dimension;
use rspigot_domain::{BiomeId, BlockPos, BlockStateId, ChunkPos, Identifier, Vec3};

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

struct Floor;

impl ChunkGenerator for Floor {
    fn generate(&self, dimension: &Dimension, pos: ChunkPos) -> ChunkColumn {
        let mut chunk = ChunkColumn::empty(pos, dimension.height, BiomeId(0));
        for x in 0..16 {
            for z in 0..16 {
                chunk.set_block(
                    BlockPos::new(pos.min_block_x() + x, 63, pos.min_block_z() + z),
                    BlockStateId(1),
                );
            }
        }
        chunk
    }

    fn spawn_point(&self, _: &Dimension) -> Vec3 {
        Vec3::new(0.5, 64.0, 0.5)
    }
}

fn profile(name: &str) -> GameProfile {
    GameProfile {
        id: PlayerId::offline(name),
        name: name.to_owned(),
        properties: Vec::new(),
    }
}

fn server(operators: Vec<String>) -> Server<Link> {
    server_with(operators, false)
}

fn server_with(operators: Vec<String>, whitelist_enabled: bool) -> Server<Link> {
    let config = GameConfig {
        view_distance: 3,
        chunk_workers: 2,
        admin: AdminLists {
            operators: operators
                .iter()
                .map(|name| ListEntry::named(name))
                .collect(),
            whitelist: Vec::new(),
            whitelist_enabled,
        },
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

fn join(server: &mut Server<Link>, name: &str) -> (PlayerId, Link) {
    let link = Link::default();
    let profile = profile(name);
    let id = profile.id;
    server.handle(Inbound::Join {
        profile,
        settings: ClientSettings::default(),
        link: link.clone(),
    });
    server.deliver();
    (id, link)
}

fn confirm_teleport(server: &mut Server<Link>, id: PlayerId, events: &[ClientEvent]) {
    for event in events {
        if let ClientEvent::Teleport { id: teleport, .. } = event {
            server.handle(Inbound::Input {
                player: id,
                input: PlayerInput::TeleportConfirm(*teleport),
            });
        }
    }
}

fn run_ticks(
    server: &mut Server<Link>,
    ticks: usize,
    links: &[(PlayerId, &Link)],
    log: &mut [Vec<ClientEvent>],
) {
    let deadline = Instant::now() + Duration::from_secs(20);
    for _ in 0..ticks {
        server.wait_for_spawn(Duration::from_millis(20));
        server.tick();
        for (index, (id, link)) in links.iter().enumerate() {
            let events = link.take();
            let batches = events
                .iter()
                .filter(|e| matches!(e, ClientEvent::ChunkBatchFinished { .. }))
                .count();
            for _ in 0..batches {
                server.handle(Inbound::Input {
                    player: *id,
                    input: PlayerInput::ChunkBatchReceived {
                        chunks_per_tick: 64.0,
                    },
                });
            }
            log[index].extend(events);
        }
        if Instant::now() > deadline {
            break;
        }
    }
}

fn chunks_received(events: &[ClientEvent]) -> Vec<ChunkPos> {
    events
        .iter()
        .filter_map(|e| match e {
            ClientEvent::Chunk { chunk, .. } => Some(chunk.pos()),
            _ => None,
        })
        .collect()
}

#[test]
fn joining_player_receives_join_then_chunks_nearest_first() {
    let mut server = server(Vec::new());
    let (id, link) = join(&mut server, "Alice");
    let first = link.take();
    assert!(matches!(first.first(), Some(ClientEvent::Join(_))));
    assert!(
        first
            .iter()
            .any(|e| matches!(e, ClientEvent::Teleport { .. }))
    );
    assert!(
        first
            .iter()
            .any(|e| matches!(e, ClientEvent::CenterChunk(p) if *p == ChunkPos::new(0, 0)))
    );
    let mut log = vec![Vec::new()];
    run_ticks(&mut server, 40, &[(id, &link)], &mut log);
    let chunks = chunks_received(&log[0]);
    assert_eq!(chunks.first(), Some(&ChunkPos::new(0, 0)));
    let unique: std::collections::HashSet<_> = chunks.iter().collect();
    assert_eq!(unique.len(), chunks.len());
    assert!(chunks.len() >= 25);
    assert_eq!(server.players_online(), 1);
}

#[test]
fn two_players_see_each_other_and_chat() {
    let mut server = server(Vec::new());
    let (alice, alice_link) = join(&mut server, "Alice");
    let (bob, bob_link) = join(&mut server, "Bob");
    let mut log = vec![Vec::new(), Vec::new()];
    confirm_teleport(&mut server, alice, &alice_link.take());
    confirm_teleport(&mut server, bob, &bob_link.take());
    run_ticks(
        &mut server,
        40,
        &[(alice, &alice_link), (bob, &bob_link)],
        &mut log,
    );
    let alice_spawns = log[0]
        .iter()
        .filter(|e| matches!(e, ClientEvent::SpawnEntity { .. }))
        .count();
    assert_eq!(alice_spawns, 1);
    server.handle(Inbound::Input {
        player: alice,
        input: PlayerInput::Move {
            position: Some(Vec3::new(2.5, 64.0, 0.5)),
            rotation: None,
            on_ground: true,
        },
    });
    server.handle(Inbound::Input {
        player: alice,
        input: PlayerInput::Chat("hello".to_owned()),
    });
    server.deliver();
    let bob_events = bob_link.take();
    assert!(
        bob_events
            .iter()
            .any(|e| matches!(e, ClientEvent::EntityMoved { to, .. } if to.x == 2.5))
    );
    assert!(
        bob_events
            .iter()
            .any(|e| matches!(e, ClientEvent::PlayerChat { message, .. } if message == "hello"))
    );
    server.handle(Inbound::Leave { player: bob });
    server.deliver();
    let alice_events = alice_link.take();
    assert!(
        alice_events
            .iter()
            .any(|e| matches!(e, ClientEvent::RemoveEntities(_)))
    );
    assert!(
        alice_events
            .iter()
            .any(|e| matches!(e, ClientEvent::PlayerInfoRemove(ids) if ids == &vec![bob]))
    );
}

#[test]
fn creative_operator_breaks_block_and_viewers_are_notified() {
    let mut server = server(vec!["alice".to_owned()]);
    let (alice, alice_link) = join(&mut server, "Alice");
    let (bob, bob_link) = join(&mut server, "Bob");
    let mut log = vec![Vec::new(), Vec::new()];
    confirm_teleport(&mut server, alice, &alice_link.take());
    confirm_teleport(&mut server, bob, &bob_link.take());
    run_ticks(
        &mut server,
        40,
        &[(alice, &alice_link), (bob, &bob_link)],
        &mut log,
    );
    server.handle(Inbound::Input {
        player: alice,
        input: PlayerInput::Command("gamemode creative".to_owned()),
    });
    let target = BlockPos::new(1, 63, 1);
    server.handle(Inbound::Input {
        player: alice,
        input: PlayerInput::Dig {
            action: DigAction::Start,
            pos: target,
            face: Direction::Up,
            sequence: 7,
        },
    });
    server.deliver();
    let alice_events = alice_link.take();
    assert!(alice_events.iter().any(|e| matches!(
        e,
        ClientEvent::GameEvent(rspigot_core::events::GameEvent::ChangeGameMode(
            GameMode::Creative
        ))
    )));
    assert!(
        alice_events
            .iter()
            .any(|e| matches!(e, ClientEvent::BlockChangedAck(7)))
    );
    let bob_events = bob_link.take();
    assert!(bob_events.iter().any(
        |e| matches!(e, ClientEvent::BlockUpdate { pos, state } if *pos == target && state.is_air())
    ));
}

#[test]
fn non_operator_survival_player_cannot_use_gamemode() {
    let mut server = server(Vec::new());
    let (alice, alice_link) = join(&mut server, "Alice");
    alice_link.take();
    server.handle(Inbound::Input {
        player: alice,
        input: PlayerInput::Command("gamemode creative".to_owned()),
    });
    server.deliver();
    let events = alice_link.take();
    assert!(
        !events
            .iter()
            .any(|e| matches!(e, ClientEvent::GameEvent(_)))
    );
    assert!(
        events
            .iter()
            .any(|e| matches!(e, ClientEvent::SystemChat { .. }))
    );
}

#[test]
fn whitelist_rejects_unlisted_player_and_admits_after_add() {
    let mut server = server_with(vec!["alice".to_owned()], true);
    let (_, bob_link) = join(&mut server, "Bob");
    assert!(!bob_link.is_connected());
    assert_eq!(server.players_online(), 0);
    let (_, alice_link) = join(&mut server, "Alice");
    assert!(alice_link.is_connected());
    server.handle(Inbound::Console("whitelist add Bob".to_owned()));
    let (_, bob_again) = join(&mut server, "Bob");
    assert!(bob_again.is_connected());
    assert_eq!(server.players_online(), 2);
}
