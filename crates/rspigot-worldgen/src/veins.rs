use rspigot_domain::{BlockStateId, ChunkPos};

use crate::buffer::ChunkBuffer;
use crate::rng::Rng;

pub(crate) const MAX_VEIN: usize = 40;

#[derive(Debug, Clone, Copy)]
pub(crate) struct Vein {
    pub(crate) count: u32,
    pub(crate) size: u32,
    pub(crate) min_y: i32,
    pub(crate) max_y: i32,
    pub(crate) triangle: bool,
    pub(crate) spread: i32,
    pub(crate) rarity: u32,
    pub(crate) in_stone: BlockStateId,
    pub(crate) in_deep: BlockStateId,
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct Hosts {
    pub(crate) stone: BlockStateId,
    pub(crate) deep: BlockStateId,
}

fn sample_y(vein: &Vein, rng: &mut Rng) -> i32 {
    let span = (vein.max_y - vein.min_y + 1).max(1) as u32;
    if vein.triangle {
        vein.min_y + ((rng.below(span) + rng.below(span)) / 2) as i32
    } else {
        vein.min_y + rng.below(span) as i32
    }
}

pub(crate) fn place(
    buffer: &mut ChunkBuffer,
    pos: ChunkPos,
    seed: u64,
    salt: u64,
    veins: &[Vein],
    hosts: Hosts,
) {
    let height = buffer.height();
    let x0 = pos.x * 16;
    let z0 = pos.z * 16;
    let mut points = [(0i32, 0i32, 0i32); MAX_VEIN];
    for dz in -1..=1 {
        for dx in -1..=1 {
            let chunk_x = pos.x + dx;
            let chunk_z = pos.z + dz;
            let mut rng = Rng::for_chunk(seed, chunk_x, chunk_z, salt);
            for vein in veins {
                let size = (vein.size as usize).clamp(1, MAX_VEIN);
                let walk_draws = 2 * (size as u64 - 1);
                for _ in 0..vein.count {
                    let ox = chunk_x * 16 + rng.below(16) as i32;
                    let oz = chunk_z * 16 + rng.below(16) as i32;
                    let oy = sample_y(vein, &mut rng);
                    let lucky = vein.rarity <= 1 || rng.below(vein.rarity) == 0;
                    let near = ox + vein.spread >= x0
                        && ox - vein.spread < x0 + 16
                        && oz + vein.spread >= z0
                        && oz - vein.spread < z0 + 16
                        && oy + vein.spread >= height.min_y
                        && oy - vein.spread <= height.max_y();
                    if !lucky || !near {
                        rng.skip(walk_draws);
                        continue;
                    }
                    points[0] = (ox, oy, oz);
                    for i in 1..size {
                        let base = points[rng.below(i as u32) as usize];
                        let (mx, my, mz) = match rng.below(6) {
                            0 => (1, 0, 0),
                            1 => (-1, 0, 0),
                            2 => (0, 1, 0),
                            3 => (0, -1, 0),
                            4 => (0, 0, 1),
                            _ => (0, 0, -1),
                        };
                        let next = (base.0 + mx, base.1 + my, base.2 + mz);
                        let inside = (next.0 - ox).abs() <= vein.spread
                            && (next.1 - oy).abs() <= vein.spread
                            && (next.2 - oz).abs() <= vein.spread;
                        points[i] = if inside { next } else { base };
                    }
                    for &(px, py, pz) in &points[..size] {
                        if px < x0 || px >= x0 + 16 || pz < z0 || pz >= z0 + 16 {
                            continue;
                        }
                        let (lx, lz) = ((px - x0) as usize, (pz - z0) as usize);
                        let current = buffer.get(lx, py, lz);
                        if current == hosts.stone {
                            buffer.set(lx, py, lz, vein.in_stone);
                        } else if current == hosts.deep {
                            buffer.set(lx, py, lz, vein.in_deep);
                        }
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rspigot_domain::world::WorldHeight;

    const STONE: BlockStateId = BlockStateId(1);
    const DEEP: BlockStateId = BlockStateId(2);
    const ORE: BlockStateId = BlockStateId(3);
    const DEEP_ORE: BlockStateId = BlockStateId(4);

    fn vein() -> Vein {
        Vein {
            count: 40,
            size: 9,
            min_y: -40,
            max_y: 40,
            triangle: false,
            spread: 2,
            rarity: 1,
            in_stone: ORE,
            in_deep: DEEP_ORE,
        }
    }

    fn filled(pos: ChunkPos, seed: u64, veins: &[Vein]) -> ChunkBuffer {
        let mut buffer = ChunkBuffer::new(WorldHeight::OVERWORLD);
        for y in -64..320 {
            buffer.fill_layer(y, if y < 0 { DEEP } else { STONE });
        }
        place(
            &mut buffer,
            pos,
            seed,
            9,
            veins,
            Hosts {
                stone: STONE,
                deep: DEEP,
            },
        );
        buffer
    }

    fn count(buffer: &ChunkBuffer, target: BlockStateId) -> usize {
        buffer.raw().iter().filter(|s| **s == target.0).count()
    }

    #[test]
    fn veins_are_placed_with_host_matching_variant() {
        let buffer = filled(ChunkPos::new(0, 0), 1, &[vein()]);
        assert!(count(&buffer, ORE) > 20);
        assert!(count(&buffer, DEEP_ORE) > 20);
        for y in 0..40 {
            for z in 0..16 {
                for x in 0..16 {
                    assert_ne!(buffer.get(x, y, z), DEEP_ORE);
                }
            }
        }
        for y in -40..-1 {
            for z in 0..16 {
                for x in 0..16 {
                    assert_ne!(buffer.get(x, y, z), ORE);
                }
            }
        }
    }

    #[test]
    fn veins_never_leave_the_host_range() {
        let buffer = filled(ChunkPos::new(3, -4), 5, &[vein()]);
        for y in 45..320 {
            assert_eq!(buffer.get(5, y, 5), STONE);
        }
        for y in -64..-45 {
            assert_eq!(buffer.get(5, y, 5), DEEP);
        }
    }

    #[test]
    fn placement_is_deterministic_and_seed_dependent() {
        let a = filled(ChunkPos::new(-2, 7), 11, &[vein()]);
        let b = filled(ChunkPos::new(-2, 7), 11, &[vein()]);
        let c = filled(ChunkPos::new(-2, 7), 12, &[vein()]);
        assert_eq!(a.raw(), b.raw());
        assert_ne!(a.raw(), c.raw());
    }

    #[test]
    fn veins_continue_across_chunk_borders() {
        let seed = 77;
        let left = filled(ChunkPos::new(0, 0), seed, &[vein()]);
        let right = filled(ChunkPos::new(1, 0), seed, &[vein()]);
        let mut adjacent_pairs = 0;
        for y in -40..40 {
            for z in 0..16 {
                if left.get(15, y, z) == ORE && right.get(0, y, z) == ORE {
                    adjacent_pairs += 1;
                }
            }
        }
        assert!(adjacent_pairs > 0);
    }

    #[test]
    fn rare_veins_appear_less_often() {
        let common = vein();
        let rare = Vein {
            rarity: 20,
            ..vein()
        };
        let a = filled(ChunkPos::new(0, 0), 3, &[common]);
        let b = filled(ChunkPos::new(0, 0), 3, &[rare]);
        assert!(count(&b, ORE) + count(&b, DEEP_ORE) < count(&a, ORE) + count(&a, DEEP_ORE));
    }
}
