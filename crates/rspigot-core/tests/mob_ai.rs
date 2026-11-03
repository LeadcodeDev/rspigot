use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use rspigot_ai::{MobAi, MobGoal};
use rspigot_blocks::{Block, BlockKind, Stone};
use rspigot_core::admin::{AdminLists, ListEntry};
use rspigot_core::chunks::ChunkManagerConfig;
use rspigot_core::config::{GameConfig, LevelData};
use rspigot_core::entities::EntitySnapshot;
use rspigot_core::events::{ClientEvent, EntityKind, Inbound, PlayerInput};
use rspigot_core::ports::{ChunkGenerator, ChunkStore, ClientLink, EntityStore, StoreError};
use rspigot_core::server::Server;
use rspigot_core::status::StatusBoard;
use rspigot_domain::chunk::ChunkColumn;
use rspigot_domain::player::{ClientSettings, GameProfile, PlayerId};
use rspigot_domain::text::Component;
use rspigot_domain::world::Dimension;
use rspigot_domain::{BiomeId, BlockPos, ChunkPos, EntityId, Identifier, Rotation, Vec3};
use rspigot_entities::EntityKind as Kind;
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

#[derive(Default)]
struct MemoryStore {
    entities: Mutex<HashMap<ChunkPos, Vec<Compound>>>,
}

impl MemoryStore {
    fn stored(&self, pos: ChunkPos) -> Vec<Compound> {
        self.entities
            .lock()
            .map(|map| map.get(&pos).cloned().unwrap_or_default())
            .unwrap_or_default()
    }
}

impl ChunkStore for MemoryStore {
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

impl EntityStore for MemoryStore {
    fn load_entities(
        &self,
        _: &Identifier,
        pos: ChunkPos,
    ) -> Result<Option<Vec<Compound>>, StoreError> {
        Ok(self
            .entities
            .lock()
            .ok()
            .and_then(|map| map.get(&pos).cloned()))
    }

    fn save_entities(
        &self,
        _: &Identifier,
        pos: ChunkPos,
        entities: &[Compound],
    ) -> Result<(), StoreError> {
        if let Ok(mut map) = self.entities.lock() {
            map.insert(pos, entities.to_vec());
        }
        Ok(())
    }
}

const POOL: ChunkPos = ChunkPos::new(0, 2);

struct Meadow;

impl ChunkGenerator for Meadow {
    fn generate(&self, dimension: &Dimension, pos: ChunkPos) -> ChunkColumn {
        let mut chunk = ChunkColumn::empty(pos, dimension.height, BiomeId(0));
        let water = BlockKind::Water.default_state();
        for x in 0..16 {
            for z in 0..16 {
                let (bx, bz) = (pos.min_block_x() + x, pos.min_block_z() + z);
                let (floor, top) = if pos == POOL { (40, 55) } else { (60, 63) };
                for y in floor..=top {
                    chunk.set_block(BlockPos::new(bx, y, bz), Stone::DEFAULT_STATE);
                }
                if pos == POOL {
                    for y in 56..=63 {
                        chunk.set_block(BlockPos::new(bx, y, bz), water);
                    }
                }
            }
        }
        chunk
    }

    fn spawn_point(&self, _: &Dimension) -> Vec3 {
        Vec3::new(0.5, 64.0, 0.5)
    }
}

fn server() -> Server<Link> {
    server_with(Arc::default())
}

fn server_with(store: Arc<MemoryStore>) -> Server<Link> {
    let config = GameConfig {
        view_distance: 3,
        simulation_distance: 3,
        chunk_workers: 2,
        chunks: ChunkManagerConfig {
            unload_delay_ticks: 5,
            autosave_interval_ticks: 0,
        },
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
        LevelData {
            day_time: 6000,
            daylight_cycle: false,
            ..LevelData::default()
        },
        store,
        Arc::new(Meadow),
        StatusBoard::new(20),
    );
    server.wait_for_spawn(Duration::from_secs(10));
    server
}

struct Client {
    id: PlayerId,
    link: Link,
    events: Vec<ClientEvent>,
}

impl Client {
    fn drain(&mut self) -> Vec<ClientEvent> {
        self.events.extend(self.link.take());
        std::mem::take(&mut self.events)
    }
}

fn input(server: &mut Server<Link>, player: PlayerId, input: PlayerInput) {
    server.handle(Inbound::Input { player, input });
    server.deliver();
}

fn pump(server: &mut Server<Link>, client: &mut Client, ticks: usize) {
    for _ in 0..ticks {
        server.wait_for_spawn(Duration::from_millis(20));
        server.tick();
        let events = client.link.take();
        for event in &events {
            match event {
                ClientEvent::ChunkBatchFinished { .. } => input(
                    server,
                    client.id,
                    PlayerInput::ChunkBatchReceived {
                        chunks_per_tick: 64.0,
                    },
                ),
                ClientEvent::Teleport { id, .. } => {
                    input(server, client.id, PlayerInput::TeleportConfirm(*id))
                }
                _ => {}
            }
        }
        client.events.extend(events);
    }
}

fn settle(server: &mut Server<Link>, client: &mut Client) {
    for _ in 0..400 {
        pump(server, client, 1);
        if server.chunk_stats().pending == 0 {
            return;
        }
        std::thread::sleep(Duration::from_millis(5));
    }
}

fn join(server: &mut Server<Link>, name: &str) -> Client {
    let link = Link::default();
    let profile = GameProfile {
        id: PlayerId::offline(name),
        name: name.to_owned(),
        properties: Vec::new(),
    };
    let id = profile.id;
    server.handle(Inbound::Join {
        profile,
        settings: ClientSettings::default(),
        link: link.clone(),
        address: std::net::Ipv4Addr::LOCALHOST.into(),
    });
    server.deliver();
    let mut client = Client {
        id,
        link,
        events: Vec::new(),
    };
    for event in client.link.take() {
        if let ClientEvent::Teleport { id: teleport, .. } = event {
            input(server, id, PlayerInput::TeleportConfirm(teleport));
        }
    }
    input(server, id, PlayerInput::Loaded);
    pump(server, &mut client, 30);
    client.drain();
    client
}

fn walk(server: &mut Server<Link>, client: &Client, position: Vec3) {
    input(
        server,
        client.id,
        PlayerInput::Move {
            position: Some(position),
            rotation: Some(Rotation::new(0.0, 0.0)),
            on_ground: true,
        },
    );
}

fn command(server: &mut Server<Link>, client: &Client, line: &str) {
    input(server, client.id, PlayerInput::Command(line.to_owned()));
}

fn only(server: &Server<Link>, kind: Kind) -> EntitySnapshot {
    let found = server.entities_of(kind);
    assert_eq!(found.len(), 1, "{found:?}");
    found[0]
}

fn summon(server: &mut Server<Link>, client: &mut Client, kind: Kind, at: Vec3) -> EntityId {
    let line = format!("/summon {} {} {} {}", kind.name(), at.x, at.y, at.z);
    command(server, client, &line);
    pump(server, client, 1);
    only(server, kind).id
}

fn ai(server: &Server<Link>, id: EntityId) -> &MobAi {
    server.mob_ai(id).expect("mob has an AI")
}

fn running(server: &Server<Link>, id: EntityId, pick: fn(&MobGoal) -> bool) -> bool {
    ai(server, id)
        .goals()
        .goals()
        .iter()
        .any(|goal| goal.is_running() && pick(goal.goal()))
}

fn horizontal(a: Vec3, b: Vec3) -> f64 {
    (a - b).horizontal_distance_sqr().sqrt()
}

#[test]
fn summoned_cow_wanders_within_stroll_distance() {
    let mut server = server();
    let mut alice = join(&mut server, "Alice");
    walk(&mut server, &alice, Vec3::new(0.5, 64.0, -6.5));
    let start = Vec3::new(4.5, 64.0, 4.5);
    let cow = summon(&mut server, &mut alice, Kind::Cow, start);
    let mut walked = 0.0f64;
    let mut strolls = 0;
    let mut previous_target = None;
    for _ in 0..1600 {
        let before = only(&server, Kind::Cow).position;
        pump(&mut server, &mut alice, 1);
        let target = ai(&server, cow).brain().navigation.target_pos();
        if target != previous_target
            && let Some(target) = target
            && running(&server, cow, |goal| matches!(goal, MobGoal::Stroll(_)))
        {
            strolls += 1;
            let from = before.block();
            assert!((target.x - from.x).abs() <= 11, "{target:?} from {from:?}");
            assert!((target.z - from.z).abs() <= 11, "{target:?} from {from:?}");
            assert!((target.y - from.y).abs() <= 7, "{target:?} from {from:?}");
        }
        previous_target = target;
        walked = walked.max(horizontal(only(&server, Kind::Cow).position, start));
    }
    assert!(strolls >= 1, "the cow never strolled");
    assert!(walked > 1.0, "the cow never moved: {walked}");
    assert_eq!(only(&server, Kind::Cow).position.y, 64.0);
    let events = alice.drain();
    let moves = events
        .iter()
        .filter(|e| matches!(e, ClientEvent::EntityMove { entity, .. } if *entity == cow))
        .count();
    assert!(moves > 10);
}

#[test]
fn cow_follows_a_player_holding_wheat_and_stops_when_the_item_changes() {
    let mut server = server();
    let mut alice = join(&mut server, "Alice");
    walk(&mut server, &alice, Vec3::new(0.5, 64.0, 0.5));
    let cow = summon(
        &mut server,
        &mut alice,
        Kind::Cow,
        Vec3::new(7.5, 64.0, 0.5),
    );
    command(&mut server, &alice, "/give Alice minecraft:wheat 4");
    pump(&mut server, &mut alice, 100);
    assert!(running(&server, cow, |goal| matches!(
        goal,
        MobGoal::Tempt(_)
    )));
    let close = horizontal(only(&server, Kind::Cow).position, Vec3::new(0.5, 64.0, 0.5));
    assert!(close < 3.5, "cow stayed {close} blocks away");
    walk(&mut server, &alice, Vec3::new(-6.5, 64.0, 0.5));
    pump(&mut server, &mut alice, 120);
    let followed = horizontal(
        only(&server, Kind::Cow).position,
        Vec3::new(-6.5, 64.0, 0.5),
    );
    assert!(followed < 3.5, "cow did not follow: {followed}");
    input(&mut server, alice.id, PlayerInput::HeldSlot(4));
    pump(&mut server, &mut alice, 8);
    assert!(!running(&server, cow, |goal| matches!(
        goal,
        MobGoal::Tempt(_)
    )));
    walk(&mut server, &alice, Vec3::new(-6.5, 64.0, 12.5));
    pump(&mut server, &mut alice, 40);
    assert!(!running(&server, cow, |goal| matches!(
        goal,
        MobGoal::Tempt(_)
    )));
    let left = horizontal(
        only(&server, Kind::Cow).position,
        Vec3::new(-6.5, 64.0, 12.5),
    );
    assert!(left > 6.0, "cow kept following: {left}");
}

fn yaw_towards(from: Vec3, to: Vec3) -> f32 {
    (f64::atan2(to.z - from.z, to.x - from.x).to_degrees() - 90.0) as f32
}

#[test]
fn mob_turns_its_head_towards_a_nearby_player() {
    let mut server = server();
    let mut alice = join(&mut server, "Alice");
    walk(&mut server, &alice, Vec3::new(0.5, 64.0, 0.5));
    let cow = summon(
        &mut server,
        &mut alice,
        Kind::Cow,
        Vec3::new(4.5, 64.0, 0.5),
    );
    alice.drain();
    let player = server.player_entity_id(alice.id).unwrap();
    let mut looked = false;
    let mut faced = false;
    let mut heads = 0;
    let mut toward = 0.0;
    for _ in 0..1500 {
        let cow_position = only(&server, Kind::Cow).position;
        let beside = Vec3::new(cow_position.x - 3.0, 64.0, cow_position.z);
        walk(&mut server, &alice, beside);
        pump(&mut server, &mut alice, 1);
        for event in alice.drain() {
            if let ClientEvent::EntityHeadRotation { entity, yaw } = event
                && entity == cow
            {
                heads += 1;
                if looked && rspigot_domain::mth::wrap_degrees(yaw - toward).abs() < 15.0 {
                    faced = true;
                }
            }
        }
        let watching = ai(&server, cow).goals().goals().iter().any(|goal| {
            goal.is_running()
                && matches!(goal.goal(), MobGoal::LookAtPlayer(look) if look.look_at() == Some(player))
        });
        looked |= watching;
        toward = yaw_towards(only(&server, Kind::Cow).position, beside);
        if faced {
            break;
        }
    }
    assert!(looked, "the cow never looked at the player");
    assert!(heads > 0, "no head rotation packets");
    assert!(faced, "the cow head never faced the player");
}

#[test]
fn cow_in_water_floats_up_to_the_surface() {
    let mut server = server();
    let mut alice = join(&mut server, "Alice");
    walk(&mut server, &alice, Vec3::new(8.5, 64.0, 26.5));
    pump(&mut server, &mut alice, 10);
    let start = Vec3::new(8.5, 57.0, 40.5);
    summon(&mut server, &mut alice, Kind::Cow, start);
    let mut highest = f64::MIN;
    for _ in 0..120 {
        pump(&mut server, &mut alice, 1);
        highest = highest.max(only(&server, Kind::Cow).position.y);
    }
    assert!(highest > 62.5, "cow stayed under water, highest {highest}");
    let cow = only(&server, Kind::Cow);
    assert!(cow.position.y > 61.5, "cow sank back to {}", cow.position.y);
}

#[test]
fn zombie_targets_survival_players_but_not_creative_ones() {
    let mut server = server();
    let mut alice = join(&mut server, "Alice");
    walk(&mut server, &alice, Vec3::new(0.5, 64.0, 0.5));
    let zombie = summon(
        &mut server,
        &mut alice,
        Kind::Zombie,
        Vec3::new(6.5, 64.0, 0.5),
    );
    let player = server.player_entity_id(alice.id).unwrap();
    let mut targeted = false;
    for _ in 0..200 {
        pump(&mut server, &mut alice, 1);
        if server.mob_target(zombie) == Some(player) {
            targeted = true;
            break;
        }
    }
    assert!(targeted, "zombie never targeted the survival player");
    command(&mut server, &alice, "/gamemode creative");
    pump(&mut server, &mut alice, 10);
    assert_eq!(server.mob_target(zombie), None);
    for _ in 0..200 {
        pump(&mut server, &mut alice, 1);
        assert_eq!(
            server.mob_target(zombie),
            None,
            "zombie targeted a creative player"
        );
    }
    let events = alice.drain();
    let paired = events.iter().any(|e| {
        matches!(e, ClientEvent::SpawnEntity { entity, kind: EntityKind::Other(_), .. } if *entity == zombie)
    });
    let moved = events
        .iter()
        .any(|e| matches!(e, ClientEvent::EntityMove { entity, .. } if *entity == zombie));
    assert!(paired && moved);
}

#[test]
fn cow_ai_is_rebuilt_after_reload_and_keeps_its_home() {
    let store = Arc::new(MemoryStore::default());
    let mut server = server_with(Arc::clone(&store));
    let mut alice = join(&mut server, "Alice");
    server.handle(Inbound::Console("tp Alice 100 64 106".to_owned()));
    pump(&mut server, &mut alice, 40);
    walk(&mut server, &alice, Vec3::new(100.5, 64.0, 106.5));
    settle(&mut server, &mut alice);
    command(
        &mut server,
        &alice,
        "/summon minecraft:cow 100.5 64 100.5 {home_pos:[I;100,64,100],home_radius:8}",
    );
    pump(&mut server, &mut alice, 5);
    let before = only(&server, Kind::Cow);
    assert_eq!(ai(&server, before.id).goals().goals().len(), 8);
    server.handle(Inbound::Console("tp Alice 2000 64 2000".to_owned()));
    pump(&mut server, &mut alice, 40);
    walk(&mut server, &alice, Vec3::new(2000.5, 64.0, 2000.5));
    pump(&mut server, &mut alice, 10);
    assert!(server.entities_of(Kind::Cow).is_empty());
    let saved = store.stored(ChunkPos::new(6, 6));
    assert_eq!(saved.len(), 1);
    assert_eq!(saved[0].get_i32("home_radius"), Some(8));
    assert_eq!(
        saved[0].get_int_array("home_pos"),
        Some(&[100, 64, 100][..])
    );
    server.handle(Inbound::Console("tp Alice 100 64 106".to_owned()));
    pump(&mut server, &mut alice, 40);
    walk(&mut server, &alice, Vec3::new(100.5, 64.0, 106.5));
    settle(&mut server, &mut alice);
    let after = only(&server, Kind::Cow);
    assert_eq!(after.uuid, before.uuid);
    assert_ne!(after.id, before.id);
    let rebuilt = ai(&server, after.id);
    assert_eq!(rebuilt.goals().goals().len(), 8);
    assert!(rebuilt.can_float());
    let home = Vec3::new(100.5, 64.0, 100.5);
    let mut farthest = 0.0f64;
    for _ in 0..3000 {
        pump(&mut server, &mut alice, 1);
        farthest = farthest.max(horizontal(only(&server, Kind::Cow).position, home));
    }
    assert!(farthest > 0.3, "cow never strolled: {farthest}");
    assert!(farthest < 10.0, "cow left its home: {farthest}");
}
