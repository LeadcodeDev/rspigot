use crate::BiomeId;
use crate::chunk::ChunkColumn;

const LCG_MULTIPLIER: i64 = 6_364_136_223_846_793_005;
const LCG_INCREMENT: i64 = 1_442_695_040_888_963_407;
const FIDDLE_SCALE: f64 = 0.9;

fn lcg_next(seed: i64, value: i64) -> i64 {
    seed.wrapping_mul(
        seed.wrapping_mul(LCG_MULTIPLIER)
            .wrapping_add(LCG_INCREMENT),
    )
    .wrapping_add(value)
}

fn fiddle(seed: i64) -> f64 {
    let value = (seed >> 24).rem_euclid(1024) as f64 / 1024.0;
    (value - 0.5) * FIDDLE_SCALE
}

fn fiddled_distance(seed: i64, x: i32, y: i32, z: i32, dx: f64, dy: f64, dz: f64) -> f64 {
    let mut value = seed;
    for coordinate in [x, y, z, x, y, z] {
        value = lcg_next(value, i64::from(coordinate));
    }
    let fx = fiddle(value);
    value = lcg_next(value, seed);
    let fy = fiddle(value);
    value = lcg_next(value, seed);
    let fz = fiddle(value);
    (dz + fz) * (dz + fz) + (dy + fy) * (dy + fy) + (dx + fx) * (dx + fx)
}

pub fn zoomed_biome(
    zoom_seed: i64,
    x: i32,
    y: i32,
    z: i32,
    mut noise: impl FnMut(i32, i32, i32) -> BiomeId,
) -> BiomeId {
    let (ax, ay, az) = (x - 2, y - 2, z - 2);
    let (px, py, pz) = (ax >> 2, ay >> 2, az >> 2);
    let fx = f64::from(ax & 3) / 4.0;
    let fy = f64::from(ay & 3) / 4.0;
    let fz = f64::from(az & 3) / 4.0;
    let mut best = 0;
    let mut best_distance = f64::INFINITY;
    for corner in 0..8 {
        let x_even = corner & 4 == 0;
        let y_even = corner & 2 == 0;
        let z_even = corner & 1 == 0;
        let distance = fiddled_distance(
            zoom_seed,
            if x_even { px } else { px + 1 },
            if y_even { py } else { py + 1 },
            if z_even { pz } else { pz + 1 },
            if x_even { fx } else { fx - 1.0 },
            if y_even { fy } else { fy - 1.0 },
            if z_even { fz } else { fz - 1.0 },
        );
        if best_distance > distance {
            best = corner;
            best_distance = distance;
        }
    }
    noise(
        if best & 4 == 0 { px } else { px + 1 },
        if best & 2 == 0 { py } else { py + 1 },
        if best & 1 == 0 { pz } else { pz + 1 },
    )
}

pub fn noise_biome(chunk: &ChunkColumn, quart_x: i32, quart_y: i32, quart_z: i32) -> BiomeId {
    let height = chunk.height();
    let min = height.min_y >> 2;
    let max = min + (height.height as i32 >> 2) - 1;
    let clamped = quart_y.clamp(min, max);
    let section = ((clamped << 2) - height.min_y) >> 4;
    chunk
        .sections()
        .get(section as usize)
        .map_or(BiomeId(0), |section| {
            section.biome(
                (quart_x & 3) as usize,
                (clamped & 3) as usize,
                (quart_z & 3) as usize,
            )
        })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ChunkPos;
    use crate::world::WorldHeight;

    #[test]
    fn zoom_picks_one_of_the_eight_surrounding_cells() {
        let mut seen = Vec::new();
        for (x, y, z) in [(0, 64, 0), (7, 70, -3), (-13, -40, 22)] {
            let picked = zoomed_biome(-4_172_144_997_891_116_543, x, y, z, |qx, qy, qz| {
                seen.push((qx, qy, qz));
                BiomeId(1)
            });
            assert_eq!(picked, BiomeId(1));
            let (px, py, pz) = ((x - 2) >> 2, (y - 2) >> 2, (z - 2) >> 2);
            let (qx, qy, qz) = *seen.last().unwrap();
            assert!((qx - px) * (qx - px) <= 1 && (qy - py) * (qy - py) <= 1);
            assert!((qz - pz) * (qz - pz) <= 1);
        }
    }

    #[test]
    fn zoom_is_deterministic_for_a_seed() {
        let lookup = |seed| {
            zoomed_biome(seed, 17, 65, -9, |x, y, z| {
                BiomeId((x * 31 + y * 7 + z) as u16)
            })
        };
        assert_eq!(lookup(42), lookup(42));
    }

    #[test]
    fn noise_biome_clamps_the_vertical_quart() {
        let mut chunk = ChunkColumn::empty(ChunkPos::new(0, 0), WorldHeight::OVERWORLD, BiomeId(3));
        chunk.sections_mut()[0].set_biome(1, 0, 2, BiomeId(9));
        assert_eq!(noise_biome(&chunk, 1, -100, 2), BiomeId(9));
        assert_eq!(noise_biome(&chunk, 1, 1000, 2), BiomeId(3));
    }
}
