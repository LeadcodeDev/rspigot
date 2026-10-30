use std::io;
use std::path::Path;
use std::sync::Arc;
use std::time::Instant;

use rspigot_anvil::{AnvilOptions, AnvilStore, Compression, DimensionLayout};
use rspigot_core::ports::{ChunkStore, StoreError};
use rspigot_data::blocks::state_with;
use rspigot_data::registries::biome_id;
use rspigot_domain::chunk::{
    Biomes, Blocks, ChunkColumn, ChunkLight, ChunkSection, Heightmap, HeightmapKind, LightArray,
    PalettedContainer,
};
use rspigot_domain::world::WorldHeight;
use rspigot_domain::{BiomeId, BlockStateId, ChunkPos, Identifier};

fn failure(reason: &str) -> StoreError {
    StoreError::Io(io::Error::other(reason.to_owned()))
}

fn state(name: &str) -> BlockStateId {
    BlockStateId(state_with(name, &[]).unwrap_or(0))
}

fn terrain_column(pos: ChunkPos) -> Result<ChunkColumn, StoreError> {
    let plains = BiomeId(biome_id("minecraft:plains").unwrap_or(0));
    let forest = BiomeId(biome_id("minecraft:forest").unwrap_or(0));
    let layers = [
        state("minecraft:bedrock"),
        state("minecraft:deepslate"),
        state("minecraft:stone"),
        state("minecraft:dirt"),
        state("minecraft:grass_block"),
        state("minecraft:gravel"),
        state("minecraft:iron_ore"),
        state("minecraft:coal_ore"),
    ];
    let mut seed =
        (pos.x as u64).wrapping_mul(0x9E37_79B9) ^ (pos.z as u64).wrapping_mul(0x85EB_CA6B) | 1;
    let mut next = move || {
        seed ^= seed << 13;
        seed ^= seed >> 7;
        seed ^= seed << 17;
        seed
    };
    let mut sections: Vec<ChunkSection> = (0..24).map(|_| ChunkSection::empty(plains)).collect();
    for (index, section) in sections.iter_mut().enumerate().take(12) {
        let values: Vec<BlockStateId> = (0..4096)
            .map(|i| {
                let ore = next() % 40 == 0;
                match (index, ore) {
                    (0, _) if i < 512 => layers[0],
                    (_, true) => layers[6 + (next() % 2) as usize],
                    (0..=5, _) => layers[1 + (next() % 2) as usize],
                    (6..=9, _) => layers[2],
                    (10, _) => layers[3],
                    _ if i < 1024 => layers[4],
                    _ => BlockStateId::AIR,
                }
            })
            .collect();
        let biomes: Vec<BiomeId> = (0..64)
            .map(|i| if i % 7 == 0 { forest } else { plains })
            .collect();
        *section = ChunkSection::from_parts(
            PalettedContainer::<BlockStateId, Blocks>::from_values(&values),
            PalettedContainer::<BiomeId, Biomes>::from_values(&biomes),
        );
    }
    let mut column = ChunkColumn::from_sections(pos, WorldHeight::OVERWORLD, sections)
        .ok_or_else(|| failure("section count"))?;
    let mut light = ChunkLight::new(24);
    for index in 0..26 {
        light.sky[index] = Some(LightArray::filled(if index > 12 { 15 } else { 0 }));
    }
    column.light = light;
    let mut surface = Heightmap::default();
    for x in 0..16 {
        for z in 0..16 {
            surface.set(x, z, 140 + ((x + z) % 5) as u16);
        }
    }
    column
        .heightmaps
        .maps
        .push((HeightmapKind::WorldSurface, surface.clone()));
    column
        .heightmaps
        .maps
        .push((HeightmapKind::MotionBlocking, surface));
    Ok(column)
}

fn report(label: &str, count: usize, started: Instant, bytes: u64) {
    let seconds = started.elapsed().as_secs_f64();
    println!(
        "{label:<28} {count:>6} chunks  {:>9.0} chunks/s  {:>8.1} MB/s",
        count as f64 / seconds,
        bytes as f64 / seconds / 1e6
    );
}

fn dir_size(path: &Path) -> u64 {
    std::fs::read_dir(path)
        .map(|entries| {
            entries
                .filter_map(Result::ok)
                .filter_map(|entry| entry.metadata().ok())
                .map(|meta| meta.len())
                .sum()
        })
        .unwrap_or(0)
}

fn join_all(
    handles: Vec<std::thread::ScopedJoinHandle<'_, Result<(), StoreError>>>,
) -> Result<(), StoreError> {
    for handle in handles {
        handle.join().map_err(|_| failure("worker panicked"))??;
    }
    Ok(())
}

fn run(compression: Compression, threads: usize) -> Result<(), StoreError> {
    let root = std::env::temp_dir().join(format!(
        "rspigot-anvil-bench-{}-{compression:?}-{threads}",
        std::process::id()
    ));
    let world = root.join("world");
    let options = AnvilOptions {
        compression,
        ..AnvilOptions::default()
    };
    let store = Arc::new(AnvilStore::open_with(
        &world,
        DimensionLayout::Spigot,
        options,
    )?);
    let dimension = Identifier::minecraft("overworld");
    let per_thread = 1024 / threads.max(1);
    let positions: Vec<Vec<ChunkPos>> = (0..threads)
        .map(|thread| {
            (0..per_thread)
                .map(|i| ChunkPos::new((thread as i32) * 32 + (i as i32 % 32), (i as i32) / 32))
                .collect()
        })
        .collect();
    let columns: Vec<Vec<ChunkColumn>> = positions
        .iter()
        .map(|list| list.iter().map(|pos| terrain_column(*pos)).collect())
        .collect::<Result<_, _>>()?;
    let total = per_thread * threads;
    let started = Instant::now();
    std::thread::scope(|scope| {
        let handles = columns
            .iter()
            .map(|list| {
                let store = Arc::clone(&store);
                let dimension = dimension.clone();
                scope.spawn(move || {
                    list.iter()
                        .try_for_each(|column| store.save(&dimension, column))
                })
            })
            .collect();
        join_all(handles)
    })?;
    store.flush()?;
    let region_dir = world.join("dimensions/minecraft/overworld/region");
    let bytes = dir_size(&region_dir);
    report(
        &format!("save {compression:?} x{threads}"),
        total,
        started,
        bytes,
    );
    println!(
        "  average stored size {} bytes",
        bytes / total.max(1) as u64
    );
    let started = Instant::now();
    std::thread::scope(|scope| {
        let handles = positions
            .iter()
            .map(|list| {
                let store = Arc::clone(&store);
                let dimension = dimension.clone();
                scope.spawn(move || {
                    for pos in list {
                        if store.load(&dimension, *pos)?.is_none() {
                            return Err(failure("missing chunk"));
                        }
                    }
                    Ok(())
                })
            })
            .collect();
        join_all(handles)
    })?;
    report(
        &format!("load {compression:?} x{threads}"),
        total,
        started,
        bytes,
    );
    drop(store);
    let _ = std::fs::remove_dir_all(&root);
    Ok(())
}

fn main() -> Result<(), StoreError> {
    for compression in [
        Compression::Zlib,
        Compression::Lz4,
        Compression::Uncompressed,
    ] {
        run(compression, 1)?;
    }
    run(Compression::Zlib, 4)?;
    run(Compression::Zlib, 8)
}
