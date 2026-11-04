use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use rspigot_core::admin::{AdminLists, ListEntry};
use rspigot_core::config::{GameConfig, LevelData};
use rspigot_core::events::{
    ClientEvent, DigAction, Direction, EntityStatus, EquipmentSlot, Hand, Inbound, PlayerInput,
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
}

fn damage_in_slot(player: &Player, hotbar: usize) -> Option<i32> {
    player.log.iter().rev().find_map(|event| match event {
        ClientEvent::InventorySlot { slot, stack, .. }
            if usize::from(*slot as u16) == 36 + hotbar =>
        {
            Some(stack.as_ref().map_or(i32::MAX, |stack| {
                stack.components.get_i32("minecraft:damage").unwrap_or(0)
            }))
        }
        _ => None,
    })
}

fn broke_main_hand_item(player: &Player) -> bool {
    player.log.iter().any(|event| {
        matches!(
            event,
            ClientEvent::EntityStatus {
                status: EntityStatus::EquipmentBreak(EquipmentSlot::MainHand),
                ..
            }
        )
    })
}

const MINING_TICKS: usize = 8;

impl World {
    fn dig(&mut self, pos: BlockPos, action: DigAction) {
        self.server.handle(Inbound::Input {
            player: self.alice.id,
            input: PlayerInput::Dig {
                action,
                pos,
                face: Direction::Up,
                sequence: 3,
            },
        });
        self.settle();
    }

    fn mine(&mut self, pos: BlockPos) {
        self.server.handle(Inbound::Input {
            player: self.alice.id,
            input: PlayerInput::Move {
                position: None,
                rotation: None,
                on_ground: true,
            },
        });
        self.dig(pos, DigAction::Start);
        for _ in 0..MINING_TICKS {
            self.server.tick();
        }
        self.dig(pos, DigAction::Finish);
    }
}

#[test]
fn mining_a_block_wears_the_tool_by_one() {
    let mut world = world();
    world.give("iron_pickaxe");
    world.select(0);
    world.mine(BlockPos::new(1, 63, 0));
    assert_eq!(damage_in_slot(&world.alice, 0), Some(1));
    world.mine(BlockPos::new(1, 63, 1));
    assert_eq!(damage_in_slot(&world.alice, 0), Some(2));
}

#[test]
fn creative_mining_does_not_wear_the_tool() {
    let mut world = world();
    world.give("iron_pickaxe");
    world.console("gamemode creative Alice");
    world.select(0);
    world.dig(BlockPos::new(1, 63, 0), DigAction::Start);
    assert_eq!(damage_in_slot(&world.alice, 0), None);
}

#[test]
fn a_block_that_breaks_at_once_does_not_wear_the_tool() {
    let mut world = world();
    world.give("poppy");
    world.give("iron_pickaxe");
    world.select(0);
    world.use_on(FLOOR, Direction::Up);
    world.select(1);
    world.dig(LOG, DigAction::Start);
    assert_eq!(damage_in_slot(&world.alice, 1), None);
}

#[test]
fn stripping_wears_the_axe_by_one() {
    let mut world = world();
    world.place_log();
    world.give("iron_axe");
    world.select(1);
    world.use_on(LOG, Direction::Up);
    assert_eq!(damage_in_slot(&world.alice, 1), Some(1));
}

#[test]
fn a_tool_breaks_on_its_last_use_and_the_others_see_it() {
    let mut world = world();
    world.give("golden_pickaxe");
    world.select(0);
    let mut mined = 0;
    'rows: for y in [63, 62] {
        for x in -2..=2 {
            for z in -2..=2 {
                if mined == 32 {
                    break 'rows;
                }
                if x == 0 && z == 0 {
                    continue;
                }
                world.mine(BlockPos::new(x, y, z));
                mined += 1;
                if mined < 32 {
                    assert!(
                        !broke_main_hand_item(&world.alice),
                        "broke after {mined} uses"
                    );
                }
            }
        }
    }
    assert_eq!(mined, 32);
    assert!(broke_main_hand_item(&world.alice));
    assert!(broke_main_hand_item(&world.bob));
    assert_eq!(damage_in_slot(&world.alice, 0), Some(i32::MAX));
}
