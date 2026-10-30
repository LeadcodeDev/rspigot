use std::hint::black_box;

use criterion::{Criterion, criterion_group, criterion_main};
use rspigot_core::ports::ChunkGenerator;
use rspigot_domain::ChunkPos;
use rspigot_domain::world::Dimension;
use rspigot_worldgen::{FlatGenerator, NoiseGenerator, VoidGenerator};

const SEED: i64 = 0x5EED_CAFE;
const EDGE: i32 = 10;

fn hundred_chunks<G: ChunkGenerator>(
    generator: &G,
    dimension: &Dimension,
    origin: ChunkPos,
) -> usize {
    let mut non_empty = 0;
    for dz in 0..EDGE {
        for dx in 0..EDGE {
            let column = generator.generate(dimension, ChunkPos::new(origin.x + dx, origin.z + dz));
            non_empty += column.sections().iter().filter(|s| !s.is_empty()).count();
        }
    }
    non_empty
}

fn bench_generators(c: &mut Criterion) {
    let overworld = Dimension::overworld();
    let nether = Dimension::nether();
    let end = Dimension::end();
    let flat = FlatGenerator::vanilla(SEED);
    let void = VoidGenerator::new(SEED);
    let noise = NoiseGenerator::new(SEED);
    let mut group = c.benchmark_group("generate_100_chunks");
    group.sample_size(20);
    group.bench_function("flat", |b| {
        b.iter(|| hundred_chunks(&flat, &overworld, black_box(ChunkPos::new(-5, -5))));
    });
    group.bench_function("void", |b| {
        b.iter(|| hundred_chunks(&void, &overworld, black_box(ChunkPos::new(-5, -5))));
    });
    group.bench_function("noise_overworld", |b| {
        b.iter(|| hundred_chunks(&noise, &overworld, black_box(ChunkPos::new(-5, -5))));
    });
    group.bench_function("noise_overworld_far", |b| {
        b.iter(|| hundred_chunks(&noise, &overworld, black_box(ChunkPos::new(4000, -2500))));
    });
    group.bench_function("noise_nether", |b| {
        b.iter(|| hundred_chunks(&noise, &nether, black_box(ChunkPos::new(-5, -5))));
    });
    group.bench_function("noise_end", |b| {
        b.iter(|| hundred_chunks(&noise, &end, black_box(ChunkPos::new(-5, -5))));
    });
    group.finish();
}

criterion_group!(benches, bench_generators);
criterion_main!(benches);
