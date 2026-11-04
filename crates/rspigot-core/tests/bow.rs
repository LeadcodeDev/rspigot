use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use rspigot_core::admin::{AdminLists, ListEntry};
use rspigot_core::config::{GameConfig, LevelData};
use rspigot_core::events::{ClientEvent, DigAction, Direction, Hand, Inbound, PlayerInput};
use rspigot_core::ports::{ChunkGenerator, ChunkStore, ClientLink, EntityStore, StoreError};
use rspigot_core::server::Server;
use rspigot_core::status::StatusBoard;
use rspigot_domain::chunk::ChunkColumn;
use rspigot_domain::player::{ClientSettings, GameProfile, PlayerId};
use rspigot_domain::text::Component;
use rspigot_domain::world::Dimension;
use rspigot_domain::{BiomeId, BlockPos, BlockStateId, ChunkPos, Identifier, Rotation, Vec3};
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

const AIM: Rotation = Rotation::new(0.0, 0.0);

fn server() -> Server<Link> {
    let config = GameConfig {
        view_distance: 3,
        chunk_workers: 2,
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

struct Archer {
    id: PlayerId,
    link: Link,
    log: Vec<ClientEvent>,
}

fn join_archer(server: &mut Server<Link>) -> Archer {
    let link = Link::default();
    let id = PlayerId::offline("Alice");
    server.handle(Inbound::Join {
        profile: GameProfile {
            id,
            name: "Alice".to_owned(),
            properties: Vec::new(),
        },
        settings: ClientSettings::default(),
        link: link.clone(),
        address: std::net::Ipv4Addr::LOCALHOST.into(),
        session: 0,
    });
    server.deliver();
    let mut archer = Archer {
        id,
        link,
        log: Vec::new(),
    };
    archer.log.extend(archer.link.take());
    for event in archer.log.clone() {
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
    run_ticks(server, &mut archer, 30);
    archer
}

fn run_ticks(server: &mut Server<Link>, archer: &mut Archer, ticks: usize) {
    for _ in 0..ticks {
        server.wait_for_spawn(Duration::from_millis(20));
        server.tick();
        let events = archer.link.take();
        let batches = events
            .iter()
            .filter(|e| matches!(e, ClientEvent::ChunkBatchFinished { .. }))
            .count();
        for _ in 0..batches {
            server.handle(Inbound::Input {
                player: archer.id,
                input: PlayerInput::ChunkBatchReceived {
                    chunks_per_tick: 64.0,
                },
            });
        }
        archer.log.extend(events);
    }
}

fn console(server: &mut Server<Link>, line: &str) {
    server.handle(Inbound::Console(line.to_owned()));
    server.deliver();
}

fn equip(server: &mut Server<Link>, archer: &mut Archer, arrows: Option<u8>) {
    console(server, "give Alice minecraft:bow");
    if let Some(count) = arrows {
        console(server, &format!("give Alice minecraft:arrow {count}"));
    }
    run_ticks(server, archer, 2);
}

fn draw(server: &mut Server<Link>, archer: &Archer) {
    server.handle(Inbound::Input {
        player: archer.id,
        input: PlayerInput::UseItem {
            hand: Hand::Main,
            sequence: 1,
            rotation: AIM,
        },
    });
}

fn release(server: &mut Server<Link>, archer: &mut Archer) {
    server.handle(Inbound::Input {
        player: archer.id,
        input: PlayerInput::Dig {
            action: DigAction::ReleaseUseItem,
            pos: BlockPos::new(0, 0, 0),
            face: Direction::Down,
            sequence: 0,
        },
    });
    server.deliver();
    archer.log.extend(archer.link.take());
}

fn shoot(server: &mut Server<Link>, archer: &mut Archer, draw_ticks: usize) {
    draw(server, archer);
    run_ticks(server, archer, draw_ticks);
    release(server, archer);
}

fn arrows_held(archer: &Archer) -> u32 {
    let arrow = ItemKind::from_name("minecraft:arrow").map(ItemKind::id);
    archer
        .log
        .iter()
        .rev()
        .find_map(|event| match event {
            ClientEvent::InventoryContent { slots, .. } => Some(
                slots
                    .iter()
                    .flatten()
                    .filter(|stack| Some(stack.item) == arrow)
                    .map(|stack| u32::from(stack.count.get()))
                    .sum(),
            ),
            _ => None,
        })
        .unwrap_or(0)
}

fn shot_sounds(archer: &Archer) -> Vec<f32> {
    archer
        .log
        .iter()
        .filter_map(|event| match event {
            ClientEvent::Sound { sound, pitch, .. } if *sound == "minecraft:entity.arrow.shoot" => {
                Some(*pitch)
            }
            _ => None,
        })
        .collect()
}

#[test]
fn a_drawn_bow_shoots_an_arrow_when_released() {
    let mut server = server();
    let mut archer = join_archer(&mut server);
    equip(&mut server, &mut archer, Some(5));
    assert_eq!(server.arrows().len(), 0);

    shoot(&mut server, &mut archer, 22);

    let arrows = server.arrows();
    assert_eq!(arrows.len(), 1);
    assert!((arrows[0].1.y - 65.52).abs() < 0.05, "{:?}", arrows[0].1);
    run_ticks(&mut server, &mut archer, 2);
    let flying = server.arrows();
    assert!(
        flying[0].1.z > 3.0,
        "the arrow flies ahead: {:?}",
        flying[0].1
    );
    assert_eq!(shot_sounds(&archer).len(), 1);
}

#[test]
fn survival_spends_one_arrow_per_shot() {
    let mut server = server();
    let mut archer = join_archer(&mut server);
    equip(&mut server, &mut archer, Some(5));
    assert_eq!(arrows_held(&archer), 5);

    shoot(&mut server, &mut archer, 22);
    run_ticks(&mut server, &mut archer, 1);
    assert_eq!(arrows_held(&archer), 4);
}

#[test]
fn a_release_before_the_minimum_draw_shoots_nothing() {
    let mut server = server();
    let mut archer = join_archer(&mut server);
    equip(&mut server, &mut archer, Some(5));

    shoot(&mut server, &mut archer, 1);
    run_ticks(&mut server, &mut archer, 1);

    assert_eq!(server.arrows().len(), 0);
    assert_eq!(arrows_held(&archer), 5);
    assert!(shot_sounds(&archer).is_empty());
}

#[test]
fn the_shot_pitch_rises_with_the_draw() {
    let mut server = server();
    let mut archer = join_archer(&mut server);
    equip(&mut server, &mut archer, Some(5));

    shoot(&mut server, &mut archer, 5);
    shoot(&mut server, &mut archer, 22);

    let pitches = shot_sounds(&archer);
    assert_eq!(pitches.len(), 2);
    assert!(pitches[1] > pitches[0], "{pitches:?}");
}

#[test]
fn a_bow_without_arrows_does_not_draw() {
    let mut server = server();
    let mut archer = join_archer(&mut server);
    equip(&mut server, &mut archer, None);

    shoot(&mut server, &mut archer, 22);

    assert_eq!(server.arrows().len(), 0);
    assert!(shot_sounds(&archer).is_empty());
}

#[test]
fn creative_shoots_without_arrows_and_without_spending_any() {
    let mut server = server();
    let mut archer = join_archer(&mut server);
    console(&mut server, "gamemode creative Alice");
    equip(&mut server, &mut archer, Some(3));

    shoot(&mut server, &mut archer, 22);
    run_ticks(&mut server, &mut archer, 1);
    assert_eq!(server.arrows().len(), 1);
    assert_eq!(arrows_held(&archer), 3);
}

#[test]
fn changing_the_held_slot_cancels_the_draw() {
    let mut server = server();
    let mut archer = join_archer(&mut server);
    equip(&mut server, &mut archer, Some(5));

    draw(&mut server, &archer);
    run_ticks(&mut server, &mut archer, 22);
    server.handle(Inbound::Input {
        player: archer.id,
        input: PlayerInput::HeldSlot(4),
    });
    server.handle(Inbound::Input {
        player: archer.id,
        input: PlayerInput::HeldSlot(0),
    });
    release(&mut server, &mut archer);

    assert_eq!(server.arrows().len(), 0);
}

#[test]
fn a_release_without_a_draw_shoots_nothing() {
    let mut server = server();
    let mut archer = join_archer(&mut server);
    equip(&mut server, &mut archer, Some(5));

    release(&mut server, &mut archer);

    assert_eq!(server.arrows().len(), 0);
}
