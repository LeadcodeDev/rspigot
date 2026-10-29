use rspigot_domain::{BlockStateId, ChunkPos};

use crate::buffer::ChunkBuffer;
use crate::grid::{Grid, WEIGHTS, trilinear};
use crate::perlin::Fractal;
use crate::rng::mix;

const CHEESE_THRESHOLD: f32 = -0.36;
const TUNNEL_BOUND: f32 = 0.075;
const BEDROCK_MARGIN: i32 = 5;

#[derive(Debug, Clone)]
pub(crate) struct Caves {
    cheese: Fractal,
    tunnel_a: Fractal,
    tunnel_b: Fractal,
    entrance: Fractal,
}

impl Caves {
    pub(crate) fn new(seed: u64) -> Self {
        let derive = |salt: u64| mix(seed ^ salt.wrapping_mul(0xD6E8_FEB8_6659_FD93));
        Self {
            cheese: Fractal::new(derive(1), 2),
            tunnel_a: Fractal::new(derive(2), 2),
            tunnel_b: Fractal::new(derive(3), 2),
            entrance: Fractal::new(derive(4), 2),
        }
    }

    pub(crate) fn limit(&self, x: i32, z: i32, height: i32, sea_level: i32) -> i32 {
        let open = height > sea_level + 1
            && self
                .entrance
                .at2(f64::from(x) * 0.022, f64::from(z) * 0.022)
                * 2.0
                > 0.62;
        if open { height } else { height - 4 }
    }

    pub(crate) fn carve(
        &self,
        buffer: &mut ChunkBuffer,
        pos: ChunkPos,
        limits: &[i32; 256],
        lava_below: i32,
        air: BlockStateId,
        lava: BlockStateId,
    ) {
        let floor = buffer.height().min_y + BEDROCK_MARGIN;
        let top = limits.iter().copied().max().unwrap_or(floor);
        if top <= floor {
            return;
        }
        let levels = ((top - floor) / 4 + 2) as usize;
        let x0 = pos.x * 16;
        let z0 = pos.z * 16;
        let grid = Grid::<3>::build(levels, |i, j, k| {
            let x = f64::from(x0 + 4 * i as i32);
            let y = f64::from(floor + 4 * j as i32);
            let z = f64::from(z0 + 4 * k as i32);
            [
                self.cheese.at3(x * 0.017, y * 0.024, z * 0.017) as f32,
                self.tunnel_a.at3(x * 0.026, y * 0.032, z * 0.026) as f32,
                self.tunnel_b
                    .at3(x * 0.026 + 91.0, y * 0.032, z * 0.026 - 57.0) as f32,
            ]
        });
        for j in 0..grid.cell_levels() {
            for k in 0..4 {
                for i in 0..4 {
                    let corners = grid.corners(i, j, k);
                    if !can_carve(&corners) {
                        continue;
                    }
                    for (dz, &wz) in WEIGHTS.iter().enumerate() {
                        for (dx, &wx) in WEIGHTS.iter().enumerate() {
                            let lx = i * 4 + dx;
                            let lz = k * 4 + dz;
                            let limit = limits[(lz << 4) | lx];
                            for (dy, &wy) in WEIGHTS.iter().enumerate() {
                                let y = floor + (j * 4 + dy) as i32;
                                if y > limit {
                                    break;
                                }
                                let [cheese, a, b] = trilinear(&corners, wx, wy, wz);
                                if cheese < CHEESE_THRESHOLD
                                    || a * a + b * b < TUNNEL_BOUND * TUNNEL_BOUND
                                {
                                    let state = if y < lava_below { lava } else { air };
                                    buffer.set(lx, y, lz, state);
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

fn can_carve(corners: &[[f32; 3]; 8]) -> bool {
    if corners.iter().any(|c| c[0] < CHEESE_THRESHOLD) {
        return true;
    }
    let crosses = |n: usize| {
        !(corners.iter().all(|c| c[n] > TUNNEL_BOUND)
            || corners.iter().all(|c| c[n] < -TUNNEL_BOUND))
    };
    crosses(1) && crosses(2)
}

#[cfg(test)]
mod tests {
    use super::*;
    use rspigot_domain::world::WorldHeight;

    const STONE: BlockStateId = BlockStateId(1);
    const LAVA: BlockStateId = BlockStateId(9);

    fn carved(seed: u64, pos: ChunkPos) -> ChunkBuffer {
        let mut buffer = ChunkBuffer::new(WorldHeight::OVERWORLD);
        for y in -64..=80 {
            buffer.fill_layer(y, STONE);
        }
        let limits = [76i32; 256];
        Caves::new(seed).carve(&mut buffer, pos, &limits, -54, BlockStateId::AIR, LAVA);
        buffer
    }

    fn air_fraction(buffer: &ChunkBuffer) -> f64 {
        let mut air = 0usize;
        let mut total = 0usize;
        for y in -59..=76 {
            for z in 0..16 {
                for x in 0..16 {
                    total += 1;
                    if buffer.get(x, y, z) != STONE {
                        air += 1;
                    }
                }
            }
        }
        air as f64 / total as f64
    }

    #[test]
    fn caves_open_a_moderate_share_of_the_rock() {
        let mut fractions = Vec::new();
        for cx in -4..4 {
            for cz in -4..4 {
                fractions.push(air_fraction(&carved(1234, ChunkPos::new(cx, cz))));
            }
        }
        let mean = fractions.iter().sum::<f64>() / fractions.len() as f64;
        assert!(mean > 0.015 && mean < 0.12, "{mean}");
        assert!(fractions.iter().any(|f| *f > 0.0));
    }

    #[test]
    fn nothing_is_carved_above_the_limit_or_into_bedrock_margin() {
        let buffer = carved(55, ChunkPos::new(2, 3));
        for z in 0..16 {
            for x in 0..16 {
                for y in 77..=80 {
                    assert_eq!(buffer.get(x, y, z), STONE);
                }
                for y in -64..-59 {
                    assert_eq!(buffer.get(x, y, z), STONE);
                }
            }
        }
    }

    #[test]
    fn deep_cave_floors_flood_with_lava() {
        let mut lava = 0;
        let mut air_deep = 0;
        for cx in -6..6 {
            for cz in -6..6 {
                let buffer = carved(9, ChunkPos::new(cx, cz));
                for y in -59..-54 {
                    for z in 0..16 {
                        for x in 0..16 {
                            if buffer.get(x, y, z) == LAVA {
                                lava += 1;
                            } else if buffer.get(x, y, z) == BlockStateId::AIR {
                                air_deep += 1;
                            }
                        }
                    }
                }
            }
        }
        assert!(lava > 0);
        assert_eq!(air_deep, 0);
    }

    #[test]
    fn carving_is_deterministic() {
        let a = carved(5, ChunkPos::new(-1, -1));
        let b = carved(5, ChunkPos::new(-1, -1));
        assert_eq!(a.raw(), b.raw());
    }

    #[test]
    fn neighbouring_chunks_agree_on_shared_borders() {
        let left = carved(31, ChunkPos::new(0, 0));
        let right = carved(31, ChunkPos::new(1, 0));
        let mut open_pairs = 0;
        let mut mismatch = 0;
        for y in -59..=76 {
            for z in 0..16 {
                let l = left.get(15, y, z) != STONE;
                let r = right.get(0, y, z) != STONE;
                if l && r {
                    open_pairs += 1;
                }
                if l != r {
                    mismatch += 1;
                }
            }
        }
        assert!(open_pairs > 0);
        assert!(mismatch < open_pairs * 3);
    }

    #[test]
    fn entrances_only_open_inland() {
        let caves = Caves::new(2);
        let mut open = 0;
        for i in 0..2000 {
            let (x, z) = (i * 7, i * 13);
            assert_eq!(caves.limit(x, z, 60, 63), 56);
            if caves.limit(x, z, 90, 63) == 90 {
                open += 1;
            }
        }
        assert!(open > 0 && open < 600, "{open}");
    }
}
