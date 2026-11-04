mod common;

use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use common::*;
use rspigot_core::chunks::ChunkManagerConfig;
use rspigot_core::config::{GameConfig, LevelData, PluginSettings};
use rspigot_core::events::Inbound;
use rspigot_core::plugins::events::{
    ChunkLoad, ChunkPopulate, ChunkUnload, ChunkUnloadPatch, EntitiesLoad, EntitiesUnload,
};
use rspigot_core::plugins::{
    Answer, ChunkRef, EventKind, EventPatch, EventPayload, HandlerSpec, ItemSpec, Question,
};
use rspigot_core::ports::{ChunkStore, EntityStore, StoreError};
use rspigot_core::server::Server;
use rspigot_core::status::StatusBoard;
use rspigot_domain::chunk::ChunkColumn;
use rspigot_domain::{BlockPos, ChunkPos, Identifier, Vec3};
use rspigot_nbt::Compound;

const FAR: ChunkPos = ChunkPos::new(12, 12);

#[derive(Default)]
struct Disk {
    chunks: Mutex<HashMap<ChunkPos, ChunkColumn>>,
    entities: Mutex<HashMap<ChunkPos, Vec<Compound>>>,
    saved: Mutex<Vec<ChunkPos>>,
}

impl Disk {
    fn saved(&self, pos: ChunkPos) -> bool {
        self.saved.lock().is_ok_and(|saved| saved.contains(&pos))
    }

    fn saves(&self, pos: ChunkPos) -> usize {
        self.saved
            .lock()
            .map_or(0, |saved| saved.iter().filter(|at| **at == pos).count())
    }

    fn saved_soon(&self, pos: ChunkPos) -> bool {
        for _ in 0..200 {
            if self.saved(pos) {
                return true;
            }
            std::thread::sleep(Duration::from_millis(10));
        }
        false
    }
}

impl ChunkStore for Disk {
    fn load(&self, _: &Identifier, pos: ChunkPos) -> Result<Option<ChunkColumn>, StoreError> {
        Ok(self
            .chunks
            .lock()
            .ok()
            .and_then(|chunks| chunks.get(&pos).cloned()))
    }

    fn save(&self, _: &Identifier, chunk: &ChunkColumn) -> Result<(), StoreError> {
        if let Ok(mut chunks) = self.chunks.lock() {
            chunks.insert(chunk.pos(), chunk.clone());
        }
        if let Ok(mut saved) = self.saved.lock() {
            saved.push(chunk.pos());
        }
        Ok(())
    }

    fn flush(&self) -> Result<(), StoreError> {
        Ok(())
    }
}

impl EntityStore for Disk {
    fn load_entities(
        &self,
        _: &Identifier,
        pos: ChunkPos,
    ) -> Result<Option<Vec<Compound>>, StoreError> {
        Ok(self
            .entities
            .lock()
            .ok()
            .and_then(|entities| entities.get(&pos).cloned()))
    }

    fn save_entities(
        &self,
        _: &Identifier,
        pos: ChunkPos,
        entities: &[Compound],
    ) -> Result<(), StoreError> {
        if let Ok(mut stored) = self.entities.lock() {
            stored.insert(pos, entities.to_vec());
        }
        Ok(())
    }
}

struct World {
    s: Scene,
    disk: Arc<Disk>,
}

fn world(handlers: Vec<HandlerSpec>) -> World {
    world_saving_every(handlers, 0)
}

fn world_saving_every(handlers: Vec<HandlerSpec>, autosave: u64) -> World {
    let config = GameConfig {
        view_distance: 2,
        chunk_workers: 2,
        spawn_protection: 0,
        chunks: ChunkManagerConfig {
            unload_delay_ticks: 3,
            autosave_interval_ticks: autosave,
        },
        plugins: PluginSettings {
            decision_ticks: if autosave > 0 { 20 } else { 1 },
            ..PluginSettings::default()
        },
        ..GameConfig::default()
    };
    let disk = Arc::new(Disk::default());
    let mut server = Server::new(
        config,
        LevelData::default(),
        Arc::clone(&disk),
        Arc::new(Floor),
        StatusBoard::new(20),
    );
    server.wait_for_spawn(Duration::from_secs(10));
    let plugin = connect_with(&mut server, 1, "tester", handlers);
    let alice = join(&mut server, "Alice");
    World {
        s: Scene {
            server,
            plugin,
            players: vec![alice],
        },
        disk,
    }
}

impl World {
    fn go(&mut self, x: f64, z: f64) {
        self.s
            .server
            .handle(Inbound::Console(format!("tp Alice {x} 64 {z}")));
        self.s.server.deliver();
        self.s.server.wait_for_spawn(Duration::from_secs(10));
        self.s.tick(1);
    }

    fn go_far(&mut self) {
        self.go(200.5, 200.5);
    }

    fn go_home(&mut self) {
        self.go(0.5, 0.5);
    }

    fn wait_unload(&mut self) {
        self.s.tick(6);
    }
}

fn chunk_of(payload: &EventPayload) -> Option<ChunkRef> {
    Some(match payload {
        EventPayload::ChunkLoad(ChunkLoad { chunk, .. })
        | EventPayload::ChunkPopulate(ChunkPopulate { chunk, .. })
        | EventPayload::ChunkUnload(ChunkUnload { chunk, .. })
        | EventPayload::EntitiesLoad(EntitiesLoad { chunk, .. })
        | EventPayload::EntitiesUnload(EntitiesUnload { chunk, .. }) => chunk.clone(),
        _ => return None,
    })
}

fn about(world: &World, pos: ChunkPos) -> Vec<EventPayload> {
    fired(&world.s.plugin)
        .into_iter()
        .filter(|event| {
            chunk_of(&event.payload).is_some_and(|chunk| chunk.x == pos.x && chunk.z == pos.z)
        })
        .map(|event| event.payload)
        .collect()
}

fn chunk_ref(pos: ChunkPos) -> ChunkRef {
    ChunkRef {
        world: "minecraft:overworld".to_owned(),
        x: pos.x,
        z: pos.z,
    }
}

fn watching() -> Vec<HandlerSpec> {
    vec![
        observe(1, EventKind::ChunkLoad),
        observe(2, EventKind::ChunkPopulate),
        observe(3, EventKind::ChunkUnload),
        observe(4, EventKind::EntitiesLoad),
        observe(5, EventKind::EntitiesUnload),
    ]
}

#[test]
fn the_spawn_chunks_load_before_any_plugin_and_reach_none() {
    let mut world = world(watching());
    world.s.tick(5);
    assert!(fired(&world.s.plugin).is_empty());
}

#[test]
fn a_generated_chunk_loads_then_populates_then_loads_its_entities() {
    let mut world = world(watching());
    world.go_far();
    assert_eq!(
        about(&world, FAR),
        [
            EventPayload::ChunkLoad(ChunkLoad {
                chunk: chunk_ref(FAR),
                new_chunk: true,
                world: "minecraft:overworld".to_owned(),
            }),
            EventPayload::ChunkPopulate(ChunkPopulate {
                chunk: chunk_ref(FAR),
                world: "minecraft:overworld".to_owned(),
            }),
            EventPayload::EntitiesLoad(EntitiesLoad {
                chunk: chunk_ref(FAR),
                entities: Vec::new(),
                world: "minecraft:overworld".to_owned(),
            }),
        ]
    );
}

#[test]
fn leaving_a_chunk_unloads_and_saves_it_with_its_entities() {
    let mut world = world(watching());
    world.go_far();
    let dropped = match world.s.ask(Question::DropItem {
        position: Vec3::new(200.5, 64.0, 200.5),
        item: ItemSpec::of("minecraft:diamond", 1),
    }) {
        Answer::Spawned(view) => view,
        other => panic!("expected a dropped item, got {other:?}"),
    };
    world.go_home();
    world.wait_unload();
    let heard = about(&world, FAR);
    assert_eq!(
        heard[3],
        EventPayload::ChunkUnload(ChunkUnload {
            chunk: chunk_ref(FAR),
            save_chunk: true,
            world: "minecraft:overworld".to_owned(),
        })
    );
    let EventPayload::EntitiesUnload(EntitiesUnload { entities, .. }) = &heard[4] else {
        panic!("expected the entities to unload, got {heard:?}");
    };
    let unloaded: Vec<_> = entities.iter().map(|entity| entity.uuid).collect();
    assert_eq!(unloaded, [dropped.uuid]);
    assert!(world.disk.saved_soon(FAR));
    world.go_far();
    let reloaded: Vec<EventPayload> = about(&world, FAR).into_iter().skip(5).collect();
    let EventPayload::ChunkLoad(ChunkLoad { new_chunk, .. }) = &reloaded[0] else {
        panic!("expected a load, got {reloaded:?}");
    };
    assert!(!new_chunk);
    let EventPayload::EntitiesLoad(EntitiesLoad { entities, .. }) = &reloaded[1] else {
        panic!("expected the entities without a populate, got {reloaded:?}");
    };
    assert_eq!(entities.len(), 1);
    assert_eq!(entities[0].uuid, dropped.uuid);
}

fn answer_unloads(world: &mut World, answered: &mut Vec<u64>, save: impl Fn(ChunkPos) -> bool) {
    let open: Vec<Fired> = asked_for(&world.s.plugin, EventKind::ChunkUnload)
        .into_iter()
        .filter(|asked| !answered.contains(&asked.id))
        .collect();
    for asked in open {
        answered.push(asked.id);
        let Some(chunk) = chunk_of(&asked.payload) else {
            continue;
        };
        let keep = save(ChunkPos::new(chunk.x, chunk.z));
        rule_on(
            &mut world.s.server,
            1,
            asked.id,
            false,
            Some(EventPatch::ChunkUnload(ChunkUnloadPatch {
                save_chunk: Some(keep),
            })),
        );
    }
}

#[test]
fn an_unload_told_not_to_save_drops_the_changes_of_its_chunk() {
    let mut world = world(vec![deciding(1, EventKind::ChunkUnload)]);
    world.go_far();
    world.go_home();
    let mut answered = Vec::new();
    for _ in 0..6 {
        world.s.tick(1);
        answer_unloads(&mut world, &mut answered, |pos| pos != FAR);
    }
    assert!(!answered.is_empty());
    assert!(world.disk.saved_soon(ChunkPos::new(13, 12)));
    std::thread::sleep(Duration::from_millis(100));
    assert!(!world.disk.saved(FAR));
    assert_eq!(
        world.s.ask(Question::Block(BlockPos::new(200, 63, 200))),
        Answer::NotFound
    );
}

#[test]
fn coming_back_while_the_unload_waits_keeps_the_chunk_and_tells_no_observer() {
    let mut world = world(vec![
        deciding(1, EventKind::ChunkUnload),
        observe(2, EventKind::ChunkUnload),
    ]);
    world.go_far();
    world.go_home();
    for _ in 0..6 {
        if !asked_for(&world.s.plugin, EventKind::ChunkUnload).is_empty() {
            break;
        }
        world.s.tick(1);
    }
    let asked = asked_for(&world.s.plugin, EventKind::ChunkUnload).len();
    assert!(asked > 0);
    world.go_far();
    let mut answered = Vec::new();
    answer_unloads(&mut world, &mut answered, |_| true);
    world.s.tick(1);
    assert!(settled_for(&world.s.plugin, EventKind::ChunkUnload).is_empty());
    assert!(matches!(
        world.s.ask(Question::Block(BlockPos::new(200, 63, 200))),
        Answer::Block(_)
    ));
    assert_eq!(
        asked_for(&world.s.plugin, EventKind::ChunkUnload).len(),
        asked
    );
}

#[test]
fn the_autosave_skips_a_chunk_whose_unload_waits() {
    let mut world = world_saving_every(vec![deciding(1, EventKind::ChunkUnload)], 2);
    world.go_far();
    world.go_home();
    let mut waiting = Vec::new();
    for _ in 0..8 {
        world.s.tick(1);
        waiting = asked_for(&world.s.plugin, EventKind::ChunkUnload)
            .into_iter()
            .filter(|asked| chunk_of(&asked.payload) == Some(chunk_ref(FAR)))
            .collect();
        if !waiting.is_empty() {
            break;
        }
    }
    assert!(!waiting.is_empty());
    world.s.act(rspigot_core::plugins::Order::Block {
        pos: BlockPos::new(200, 63, 200),
        order: rspigot_core::plugins::BlockOrder::Set("minecraft:dirt".to_owned()),
    });
    assert!(matches!(
        world.s.ask(Question::Block(BlockPos::new(200, 63, 200))),
        Answer::Block(state) if state.kind == "minecraft:dirt"
    ));
    std::thread::sleep(Duration::from_millis(200));
    let before = world.disk.saves(FAR);
    world.s.tick(4);
    std::thread::sleep(Duration::from_millis(200));
    assert_eq!(world.disk.saves(FAR), before);
    let mut answered = Vec::new();
    answer_unloads(&mut world, &mut answered, |pos| pos != FAR);
    world.s.tick(4);
    std::thread::sleep(Duration::from_millis(100));
    assert_eq!(world.disk.saves(FAR), before);
}
