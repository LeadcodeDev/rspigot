use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use rspigot_blocks::BlockKind;
use rspigot_core::admin::{AdminLists, ListEntry};
use rspigot_core::config::{GameConfig, LevelData};
use rspigot_core::events::{
    ClientEvent, DigAction, Direction, Hand, Inbound, InputState, PlayerInput,
};
use rspigot_core::ports::{ChunkGenerator, ChunkStore, ClientLink, EntityStore, StoreError};
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

const FLOOR: BlockPos = BlockPos::new(2, 63, 2);
const LOG: BlockPos = BlockPos::new(2, 64, 2);
const SIDE_LOG: BlockPos = BlockPos::new(3, 64, 2);
const STRIP_SOUND: &str = "minecraft:item.axe.strip";

struct Player {
    id: PlayerId,
    link: Link,
    log: Vec<ClientEvent>,
}

fn server() -> Server<Link> {
    let config = GameConfig {
        view_distance: 3,
        chunk_workers: 2,
        spawn_protection: 0,
        admin: AdminLists {
            operators: vec![ListEntry::named("Alice")],
            whitelist: Vec::new(),
            whitelist_enabled: false,
            ..AdminLists::default()
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
    let mut player = Player {
        id,
        link,
        log: Vec::new(),
    };
    player.log.extend(player.link.take());
    for event in player.log.clone() {
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
    player
}

fn run_ticks(server: &mut Server<Link>, players: &mut [&mut Player], ticks: usize) {
    let deadline = Instant::now() + Duration::from_secs(20);
    for _ in 0..ticks {
        server.wait_for_spawn(Duration::from_millis(20));
        server.tick();
        for player in players.iter_mut() {
            let events = player.link.take();
            let batches = events
                .iter()
                .filter(|e| matches!(e, ClientEvent::ChunkBatchFinished { .. }))
                .count();
            for _ in 0..batches {
                server.handle(Inbound::Input {
                    player: player.id,
                    input: PlayerInput::ChunkBatchReceived {
                        chunks_per_tick: 64.0,
                    },
                });
            }
            player.log.extend(events);
        }
        if Instant::now() > deadline {
            break;
        }
    }
}

struct World {
    server: Server<Link>,
    alice: Player,
    bob: Player,
}

fn world() -> World {
    let mut server = server();
    let mut alice = join(&mut server, "Alice");
    let mut bob = join(&mut server, "Bob");
    run_ticks(&mut server, &mut [&mut alice, &mut bob], 30);
    World { server, alice, bob }
}

impl World {
    fn settle(&mut self) {
        self.server.deliver();
        run_ticks(&mut self.server, &mut [&mut self.alice, &mut self.bob], 2);
    }

    fn console(&mut self, line: &str) {
        self.server.handle(Inbound::Console(line.to_owned()));
        self.settle();
    }

    fn select(&mut self, slot: u8) {
        self.server.handle(Inbound::Input {
            player: self.alice.id,
            input: PlayerInput::HeldSlot(slot),
        });
    }

    fn use_on(&mut self, pos: BlockPos, face: Direction) {
        self.server.handle(Inbound::Input {
            player: self.alice.id,
            input: PlayerInput::UseItemOn {
                hand: Hand::Main,
                pos,
                face,
                cursor: Vec3::new(0.5, 1.0, 0.5),
                inside_block: false,
                sequence: 7,
            },
        });
        self.settle();
    }

    fn give(&mut self, item: &str) {
        self.console(&format!("give Alice minecraft:{item}"));
    }

    fn place_log(&mut self) {
        self.console("give Alice minecraft:oak_log 2");
        self.select(0);
        self.use_on(FLOOR, Direction::Up);
    }

    fn sneak(&mut self, shift: bool) {
        self.server.handle(Inbound::Input {
            player: self.alice.id,
            input: PlayerInput::Input(InputState {
                shift,
                ..InputState::default()
            }),
        });
    }
}

fn state_of(name: &str, axis: &str) -> BlockStateId {
    BlockKind::from_name(name)
        .expect("known block")
        .default_state()
        .with_property("axis", axis)
        .expect("axis")
}

fn last_update(player: &Player, pos: BlockPos) -> Option<BlockStateId> {
    player.log.iter().rev().find_map(|event| match event {
        ClientEvent::BlockUpdate { pos: at, state } if *at == pos => Some(*state),
        _ => None,
    })
}

fn heard_strip(player: &Player) -> bool {
    player
        .log
        .iter()
        .any(|event| matches!(event, ClientEvent::Sound { sound, .. } if *sound == STRIP_SOUND))
}

fn acked(player: &Player) -> bool {
    player
        .log
        .iter()
        .any(|event| matches!(event, ClientEvent::BlockChangedAck(7)))
}

#[test]
fn an_axe_strips_a_log_and_the_others_hear_it() {
    let mut world = world();
    world.place_log();
    assert_eq!(
        last_update(&world.alice, LOG),
        Some(state_of("oak_log", "y"))
    );

    world.give("iron_axe");
    world.select(1);
    world.use_on(LOG, Direction::Up);

    let stripped = Some(state_of("stripped_oak_log", "y"));
    assert_eq!(last_update(&world.alice, LOG), stripped);
    assert_eq!(last_update(&world.bob, LOG), stripped);
    assert!(heard_strip(&world.bob));
    assert!(
        !heard_strip(&world.alice),
        "the stripping player predicts the sound"
    );
    assert!(acked(&world.alice));
}

#[test]
fn stripping_keeps_the_axis_a_log_was_placed_with() {
    let mut world = world();
    world.place_log();
    world.select(0);
    world.use_on(LOG, Direction::East);
    assert_eq!(
        last_update(&world.alice, SIDE_LOG),
        Some(state_of("oak_log", "x"))
    );

    world.give("iron_axe");
    world.select(0);
    world.use_on(SIDE_LOG, Direction::Up);

    assert_eq!(
        last_update(&world.alice, SIDE_LOG),
        Some(state_of("stripped_oak_log", "x"))
    );
}

#[test]
fn an_item_that_is_not_an_axe_does_not_strip() {
    let mut world = world();
    world.place_log();
    world.give("stick");
    world.select(1);
    world.use_on(LOG, Direction::Up);

    assert_eq!(
        last_update(&world.alice, LOG),
        Some(state_of("oak_log", "y"))
    );
    assert!(!heard_strip(&world.bob));
}

#[test]
fn an_axe_does_nothing_to_a_block_that_cannot_be_stripped() {
    let mut world = world();
    world.give("iron_axe");
    world.select(0);
    world.use_on(FLOOR, Direction::Up);

    assert_eq!(last_update(&world.alice, FLOOR), None);
    assert!(!heard_strip(&world.bob));
    assert!(acked(&world.alice));
}

#[test]
fn adventure_mode_cannot_strip() {
    let mut world = world();
    world.place_log();
    world.give("iron_axe");
    world.console("gamemode adventure Alice");
    world.select(1);
    world.use_on(LOG, Direction::Up);

    assert_eq!(
        last_update(&world.alice, LOG),
        Some(state_of("oak_log", "y"))
    );
    assert!(!heard_strip(&world.bob));
}

#[test]
fn a_shield_in_the_offhand_blocks_stripping_unless_sneaking() {
    let mut world = world();
    world.place_log();
    world.give("iron_axe");
    world.give("shield");
    world.select(2);
    world.server.handle(Inbound::Input {
        player: world.alice.id,
        input: PlayerInput::Dig {
            action: DigAction::SwapHands,
            pos: BlockPos::new(0, 0, 0),
            face: Direction::Down,
            sequence: 0,
        },
    });
    world.select(1);
    world.use_on(LOG, Direction::Up);
    assert_eq!(
        last_update(&world.alice, LOG),
        Some(state_of("oak_log", "y"))
    );

    world.sneak(true);
    world.use_on(LOG, Direction::Up);
    assert_eq!(
        last_update(&world.alice, LOG),
        Some(state_of("stripped_oak_log", "y"))
    );
}
