use rspigot_data::spawns::Spawner;
use rspigot_entities::EntityKind;

use super::placement::{adjust_spawn_position, is_spawn_position_ok};
use super::rules::{
    check_spawn_obstruction, check_spawn_rules, mob_check_spawn_rules, no_collision,
};
use super::{
    SpawnGroupData, SpawnLevel, SpawnReason, pick_spawner, sample_count, spawn_aabb, spawner_kind,
};
use crate::entity::Aabb;
use crate::random::RandomSource;
use crate::{BlockPos, ChunkPos, Vec3};

const ATTEMPTS: i32 = 4;
const WANDER: i32 = 5;

pub trait GenerationHost: SpawnLevel + RandomSource {
    fn naturally_spawns(&self, kind: EntityKind) -> bool;
    fn spawn_mob(
        &mut self,
        kind: EntityKind,
        position: Vec3,
        yaw: f32,
        group: Option<SpawnGroupData>,
    ) -> Option<SpawnGroupData>;
}

pub fn top_non_colliding_pos(
    level: &impl SpawnLevel,
    kind: EntityKind,
    x: i32,
    z: i32,
) -> BlockPos {
    let heightmap = kind.mob_spawn().map_or(
        rspigot_entities::SpawnHeightmap::MotionBlockingNoLeaves,
        |spawn| spawn.heightmap(),
    );
    let mut pos = BlockPos::new(x, level.top_height(heightmap, x, z), z);
    if level.dimension().has_ceiling {
        loop {
            pos = pos.offset(0, -1, 0);
            if level.block_or_air(pos).is_air() {
                break;
            }
        }
        loop {
            pos = pos.offset(0, -1, 0);
            if !(level.block_or_air(pos).is_air() && pos.y > level.height().min_y) {
                break;
            }
        }
    }
    adjust_spawn_position(level, kind, pos)
}

fn clamp(value: f64, min: f64, max: f64) -> f64 {
    if value < min { min } else { value.min(max) }
}

pub fn spawn_mobs_for_chunk_generation(
    host: &mut impl GenerationHost,
    chunk: ChunkPos,
    spawners: &'static [Spawner],
    probability: f32,
    random: &mut impl RandomSource,
) -> usize {
    if spawners.is_empty() {
        return 0;
    }
    let mut spawned = 0;
    let min_x = chunk.min_block_x();
    let min_z = chunk.min_block_z();
    while random.next_float() < probability {
        let Some(picked) = pick_spawner(spawners, random) else {
            continue;
        };
        let count = sample_count(picked.count, random);
        let Some(kind) = spawner_kind(picked) else {
            continue;
        };
        let mut group: Option<SpawnGroupData> = None;
        let mut x = min_x + random.next_int(16);
        let mut z = min_z + random.next_int(16);
        let (start_x, start_z) = (x, z);
        for _ in 0..count {
            let mut success = false;
            let mut attempt = 0;
            while !success && attempt < ATTEMPTS {
                attempt += 1;
                let top = top_non_colliding_pos(host, kind, x, z);
                if kind.summonable() && is_spawn_position_ok(host, kind, top) {
                    let width = f64::from(kind.dimensions().width);
                    let cx = clamp(
                        f64::from(x),
                        f64::from(min_x) + width,
                        f64::from(min_x) + 16.0 - width,
                    );
                    let cz = clamp(
                        f64::from(z),
                        f64::from(min_z) + width,
                        f64::from(min_z) + 16.0 - width,
                    );
                    let y = f64::from(top.y);
                    if !no_collision(host, &spawn_aabb(kind, cx, y, cz)) {
                        continue;
                    }
                    let at = Vec3::new(cx, y, cz).block();
                    if !check_spawn_rules(host, kind, SpawnReason::ChunkGeneration, at) {
                        continue;
                    }
                    if !host.naturally_spawns(kind) {
                        continue;
                    }
                    let yaw = random.next_float() * 360.0;
                    let position = Vec3::new(cx, y, cz);
                    let dimensions = kind.dimensions();
                    let bounds = Aabb::of_size(position, dimensions.width, dimensions.height);
                    if mob_check_spawn_rules(host, kind, position)
                        && check_spawn_obstruction(host, kind, &bounds)
                    {
                        group = host.spawn_mob(kind, position, yaw, group);
                        spawned += 1;
                        success = true;
                    }
                }
                x += random.next_int(WANDER) - random.next_int(WANDER);
                z += random.next_int(WANDER) - random.next_int(WANDER);
                while x < min_x || x >= min_x + 16 || z < min_z || z >= min_z + 16 {
                    x = start_x + random.next_int(WANDER) - random.next_int(WANDER);
                    z = start_z + random.next_int(WANDER) - random.next_int(WANDER);
                }
            }
        }
    }
    spawned
}

#[cfg(test)]
mod tests {
    use rspigot_blocks::{Block, GrassBlock, Stone};

    use super::*;
    use crate::random::LegacyRandom;
    use crate::spawn::testing::Level;

    impl GenerationHost for Level {
        fn naturally_spawns(&self, kind: EntityKind) -> bool {
            super::super::natural::NaturalSpawnHost::naturally_spawns(self, kind)
        }

        fn spawn_mob(
            &mut self,
            kind: EntityKind,
            position: Vec3,
            yaw: f32,
            group: Option<SpawnGroupData>,
        ) -> Option<SpawnGroupData> {
            super::super::natural::NaturalSpawnHost::spawn_mob(
                self,
                kind,
                position,
                yaw,
                SpawnReason::ChunkGeneration,
                group,
            )
            .1
        }
    }

    fn plains() -> &'static [Spawner] {
        let plains = rspigot_data::registries::biome_id("minecraft:plains").unwrap_or(0);
        rspigot_data::spawns::biome(plains).creature
    }

    #[test]
    fn top_position_is_the_first_free_block_above_the_surface() {
        let level = Level::floor(GrassBlock::DEFAULT_STATE, 63);
        assert_eq!(
            top_non_colliding_pos(&level, EntityKind::Cow, 5, 9),
            BlockPos::new(5, 64, 9)
        );
    }

    #[test]
    fn grass_chunks_get_their_animals_inside_the_chunk() {
        let mut level = Level::floor(GrassBlock::DEFAULT_STATE, 63);
        let chunk = ChunkPos::new(3, -2);
        let mut total = 0;
        for seed in 0..40 {
            let mut random =
                LegacyRandom::decoration(seed, chunk.min_block_x(), chunk.min_block_z());
            total += spawn_mobs_for_chunk_generation(&mut level, chunk, plains(), 0.1, &mut random);
        }
        assert!(total > 0);
        assert_eq!(level.spawned.len(), total);
        for (kind, position, yaw) in &level.spawned {
            assert!(kind.extends("Animal"));
            assert_eq!(position.chunk(), chunk);
            assert_eq!(position.y, 64.0);
            assert!((0.0..360.0).contains(yaw));
        }
    }

    #[test]
    fn stone_chunks_and_dark_chunks_get_no_animals() {
        let mut level = Level::floor(Stone::DEFAULT_STATE, 63);
        let chunk = ChunkPos::new(0, 0);
        let mut random = LegacyRandom::new(7);
        assert_eq!(
            spawn_mobs_for_chunk_generation(
                &mut level,
                chunk,
                plains(),
                0.5,
                &mut LegacyRandom::new(3)
            ),
            0
        );
        let mut dark = Level::floor(GrassBlock::DEFAULT_STATE, 63);
        dark.sky = 0;
        assert_eq!(
            spawn_mobs_for_chunk_generation(&mut dark, chunk, plains(), 0.5, &mut random),
            0
        );
        assert_eq!(
            spawn_mobs_for_chunk_generation(&mut dark, chunk, &[], 1.0, &mut random),
            0
        );
    }
}
