use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use rspigot_blocks::BlockKind;
use rspigot_core::admin::{AdminLists, ListEntry};
use rspigot_core::config::{GameConfig, LevelData};
use rspigot_core::events::{ClientEvent, Direction, Hand, Inbound, PlayerInput};
use rspigot_core::ports::{ChunkGenerator, ChunkStore, ClientLink, EntityStore, StoreError};
use rspigot_core::server::Server;
use rspigot_core::status::StatusBoard;
use rspigot_domain::chunk::ChunkColumn;
use rspigot_domain::player::{ClientSettings, GameProfile, PlayerId};
use rspigot_domain::text::Component;
use rspigot_domain::world::Dimension;
use rspigot_domain::{BiomeId, BlockPos, BlockStateId, ChunkPos, Identifier, Vec3};
use rspigot_items::ItemKind;
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
const SOIL: BlockPos = BlockPos::new(2, 64, 2);
const ABOVE: BlockPos = BlockPos::new(2, 65, 2);

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
}

fn soil(name: &str) -> BlockStateId {
    BlockKind::from_name(name)
        .expect("known block")
        .default_state()
}

fn farmland() -> BlockStateId {
    soil("farmland")
        .with_property("moisture", "0")
        .expect("moisture")
}

fn last_update(player: &Player, pos: BlockPos) -> Option<BlockStateId> {
    player.log.iter().rev().find_map(|event| match event {
        ClientEvent::BlockUpdate { pos: at, state } if *at == pos => Some(*state),
        _ => None,
    })
}

fn heard(player: &Player, name: &str) -> bool {
    player
        .log
        .iter()
        .any(|event| matches!(event, ClientEvent::Sound { sound, .. } if *sound == name))
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

impl World {
    fn place_soil(&mut self, tool: &str, block: &str) {
        self.give(tool);
        self.give(block);
        self.select(1);
        self.use_on(FLOOR, Direction::Up);
        self.select(0);
        assert_eq!(last_update(&self.alice, SOIL), Some(soil(block)));
    }
}

#[test]
fn a_hoe_tills_dirt_and_the_others_hear_it() {
    let mut world = world();
    world.place_soil("iron_hoe", "dirt");
    world.use_on(SOIL, Direction::Up);

    assert_eq!(last_update(&world.alice, SOIL), Some(farmland()));
    assert_eq!(last_update(&world.bob, SOIL), Some(farmland()));
    assert!(heard(&world.bob, "minecraft:item.hoe.till"));
    assert!(!heard(&world.alice, "minecraft:item.hoe.till"));
    assert_eq!(damage_in_slot(&world.alice, 0), Some(1));
}

#[test]
fn a_hoe_does_not_till_from_below() {
    let mut world = world();
    world.place_soil("iron_hoe", "dirt");
    world.use_on(SOIL, Direction::Down);

    assert_eq!(last_update(&world.alice, SOIL), Some(soil("dirt")));
    assert!(!heard(&world.bob, "minecraft:item.hoe.till"));
    assert_eq!(damage_in_slot(&world.alice, 0), None);
}

#[test]
fn a_hoe_does_not_till_soil_with_a_block_above() {
    let mut world = world();
    world.place_soil("iron_hoe", "dirt");
    world.give("stone");
    world.select(1);
    world.use_on(SOIL, Direction::Up);
    assert_eq!(last_update(&world.alice, ABOVE), Some(soil("stone")));

    world.select(0);
    world.use_on(SOIL, Direction::Up);
    assert_eq!(last_update(&world.alice, SOIL), Some(soil("dirt")));
}

#[test]
fn a_hoe_turns_coarse_dirt_into_dirt() {
    let mut world = world();
    world.place_soil("iron_hoe", "coarse_dirt");
    world.use_on(SOIL, Direction::Up);

    assert_eq!(last_update(&world.alice, SOIL), Some(soil("dirt")));
    assert_eq!(damage_in_slot(&world.alice, 0), Some(1));
}

#[test]
fn a_hoe_turns_rooted_dirt_into_dirt_and_pops_its_roots() {
    let mut world = world();
    world.place_soil("iron_hoe", "rooted_dirt");
    assert!(world.server.ground_items().is_empty());
    world.use_on(SOIL, Direction::Up);
    world.settle();

    assert_eq!(last_update(&world.alice, SOIL), Some(soil("dirt")));
    let roots = ItemKind::from_name("minecraft:hanging_roots").map(ItemKind::id);
    let dropped = world.server.ground_items();
    assert_eq!(dropped.len(), 1);
    assert_eq!(Some(dropped[0].1.item), roots);
}

#[test]
fn a_shovel_flattens_grass_into_a_path_and_wears() {
    let mut world = world();
    world.place_soil("iron_shovel", "grass_block");
    world.use_on(SOIL, Direction::Up);

    assert_eq!(last_update(&world.alice, SOIL), Some(soil("dirt_path")));
    assert!(heard(&world.bob, "minecraft:item.shovel.flatten"));
    assert_eq!(damage_in_slot(&world.alice, 0), Some(1));
}

#[test]
fn a_hoe_does_nothing_in_adventure_mode() {
    let mut world = world();
    world.place_soil("iron_hoe", "dirt");
    world.console("gamemode adventure Alice");
    world.use_on(SOIL, Direction::Up);

    assert_eq!(last_update(&world.alice, SOIL), Some(soil("dirt")));
}

fn level_event(player: &Player, event: i32, pos: BlockPos) -> bool {
    player.log.iter().any(|candidate| {
        matches!(
            candidate,
            ClientEvent::LevelEvent { event: seen, pos: at, .. } if *seen == event && *at == pos
        )
    })
}

fn any_level_event(player: &Player) -> bool {
    player
        .log
        .iter()
        .any(|event| matches!(event, ClientEvent::LevelEvent { .. }))
}

#[test]
fn scraping_copper_steps_it_back_and_only_the_others_see_the_particles() {
    let mut world = world();
    world.place_soil("iron_axe", "exposed_copper");
    world.use_on(SOIL, Direction::Up);

    assert_eq!(last_update(&world.alice, SOIL), Some(soil("copper_block")));
    assert!(heard(&world.bob, "minecraft:item.axe.scrape"));
    assert!(level_event(&world.bob, 3005, SOIL));
    assert!(!any_level_event(&world.alice));
    assert_eq!(damage_in_slot(&world.alice, 0), Some(1));
}

#[test]
fn removing_wax_gives_back_the_unwaxed_block() {
    let mut world = world();
    world.place_soil("iron_axe", "waxed_oxidized_copper");
    world.use_on(SOIL, Direction::Up);

    assert_eq!(
        last_update(&world.alice, SOIL),
        Some(soil("oxidized_copper"))
    );
    assert!(heard(&world.bob, "minecraft:item.axe.wax_off"));
    assert!(level_event(&world.bob, 3004, SOIL));
    assert_eq!(damage_in_slot(&world.alice, 0), Some(1));
}

#[test]
fn a_stripping_axe_makes_no_particles() {
    let mut world = world();
    world.place_soil("iron_axe", "oak_log");
    world.use_on(SOIL, Direction::Up);

    assert!(heard(&world.bob, "minecraft:item.axe.strip"));
    assert!(!any_level_event(&world.bob));
}

#[test]
fn copper_doors_are_left_alone_for_now() {
    let mut world = world();
    world.give("iron_axe");
    world.give("exposed_copper_door");
    world.select(1);
    world.use_on(FLOOR, Direction::Up);
    world.select(0);
    world.use_on(SOIL, Direction::Up);

    assert!(!heard(&world.bob, "minecraft:item.axe.scrape"));
    assert!(!any_level_event(&world.bob));
    assert_eq!(damage_in_slot(&world.alice, 0), None);
}
