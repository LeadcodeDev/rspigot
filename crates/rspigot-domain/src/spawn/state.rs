use rspigot_entities::{EntityKind, MobCategory};
use rustc_hash::FxHashMap;

use crate::{BlockPos, ChunkPos, Vec3};

pub const MAGIC_NUMBER: i32 = 289;
pub const SPAWN_DISTANCE_CHUNK: i32 = 8;
pub const INSCRIBED_SQUARE_SPAWN_DISTANCE_CHUNK: i32 = 5;
pub const SPAWN_DISTANCE_BLOCK_SQR: f64 = 16_384.0;
const CATEGORY_COUNT: usize = MobCategory::ALL.len();

pub const SPAWNING_CATEGORIES: [MobCategory; CATEGORY_COUNT - 1] = [
    MobCategory::Monster,
    MobCategory::Creature,
    MobCategory::Ambient,
    MobCategory::Axolotls,
    MobCategory::UndergroundWaterCreature,
    MobCategory::WaterCreature,
    MobCategory::WaterAmbient,
];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CategorySettings {
    pub limit: i32,
    pub ticks_per: i64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SpawnSettings {
    pub monster: CategorySettings,
    pub creature: CategorySettings,
    pub ambient: CategorySettings,
    pub axolotls: CategorySettings,
    pub underground_water_creature: CategorySettings,
    pub water_creature: CategorySettings,
    pub water_ambient: CategorySettings,
}

impl Default for SpawnSettings {
    fn default() -> Self {
        let every = |limit| CategorySettings {
            limit,
            ticks_per: 1,
        };
        Self {
            monster: every(70),
            creature: CategorySettings {
                limit: 10,
                ticks_per: 400,
            },
            ambient: every(15),
            axolotls: every(5),
            underground_water_creature: every(5),
            water_creature: every(5),
            water_ambient: every(20),
        }
    }
}

impl SpawnSettings {
    pub fn category(&self, category: MobCategory) -> Option<CategorySettings> {
        Some(match category {
            MobCategory::Monster => self.monster,
            MobCategory::Creature => self.creature,
            MobCategory::Ambient => self.ambient,
            MobCategory::Axolotls => self.axolotls,
            MobCategory::UndergroundWaterCreature => self.underground_water_creature,
            MobCategory::WaterCreature => self.water_creature,
            MobCategory::WaterAmbient => self.water_ambient,
            MobCategory::Misc => return None,
        })
    }
}

pub fn spawn_friendlies(settings: &SpawnSettings, game_time: i64) -> bool {
    let ticks = settings.creature.ticks_per;
    ticks != 0 && game_time % ticks == 0
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SpawnPlayer {
    pub position: Vec3,
    pub spectator: bool,
}

pub fn chunk_center_distance_sqr(chunk: ChunkPos, position: Vec3) -> f64 {
    let dx = f64::from(chunk.min_block_x() + 8) - position.x;
    let dz = f64::from(chunk.min_block_z() + 8) - position.z;
    dx * dx + dz * dz
}

pub fn natural_spawn_level(players: &[SpawnPlayer], chunk: ChunkPos) -> Option<i32> {
    players
        .iter()
        .map(|player| player.position.chunk().chebyshev_distance(chunk))
        .min()
}

pub fn player_close_for_spawning(player: &SpawnPlayer, chunk: ChunkPos, distance_sqr: f64) -> bool {
    !player.spectator && chunk_center_distance_sqr(chunk, player.position) < distance_sqr
}

pub fn any_player_close_for_spawning(
    players: &[SpawnPlayer],
    chunk: ChunkPos,
    reduced_range: Option<i32>,
) -> bool {
    if let Some(range) = reduced_range {
        match natural_spawn_level(players, chunk) {
            Some(level) if level <= INSCRIBED_SQUARE_SPAWN_DISTANCE_CHUNK => return true,
            Some(level) if level > SPAWN_DISTANCE_CHUNK => return false,
            None => return false,
            _ => {}
        }
        let blocks = f64::from(range << 4);
        return players
            .iter()
            .any(|player| player_close_for_spawning(player, chunk, blocks * blocks));
    }
    players
        .iter()
        .any(|player| player_close_for_spawning(player, chunk, SPAWN_DISTANCE_BLOCK_SQR))
}

pub fn effective_mob_spawn_range(mob_spawn_range: i32, view_distance: i32) -> i32 {
    mob_spawn_range.min(view_distance).min(SPAWN_DISTANCE_CHUNK)
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct PotentialCalculator {
    charges: Vec<(BlockPos, f64)>,
}

impl PotentialCalculator {
    pub fn add_charge(&mut self, pos: BlockPos, charge: f64) {
        if charge != 0.0 {
            self.charges.push((pos, charge));
        }
    }

    pub fn potential_energy_change(&self, pos: BlockPos, charge: f64) -> f64 {
        if charge == 0.0 {
            return 0.0;
        }
        let total: f64 = self
            .charges
            .iter()
            .map(|(at, charge)| {
                let (dx, dy, dz) = (
                    f64::from(at.x - pos.x),
                    f64::from(at.y - pos.y),
                    f64::from(at.z - pos.z),
                );
                let distance = dx * dx + dy * dy + dz * dz;
                if distance == 0.0 {
                    f64::INFINITY
                } else {
                    charge / distance.sqrt()
                }
            })
            .fold(0.0, |sum, change| sum + change);
        total * charge
    }
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct LocalMobCaps {
    players: Vec<SpawnPlayer>,
    near: FxHashMap<ChunkPos, Vec<usize>>,
    counts: Vec<Option<[i32; CATEGORY_COUNT]>>,
}

impl LocalMobCaps {
    pub fn new(players: Vec<SpawnPlayer>) -> Self {
        let counts = vec![None; players.len()];
        Self {
            players,
            near: FxHashMap::default(),
            counts,
        }
    }

    fn players_near(&mut self, chunk: ChunkPos) -> &[usize] {
        let players = &self.players;
        self.near.entry(chunk).or_insert_with(|| {
            if natural_spawn_level(players, chunk).is_none_or(|level| level > SPAWN_DISTANCE_CHUNK)
            {
                return Vec::new();
            }
            players
                .iter()
                .enumerate()
                .filter(|(_, player)| {
                    player_close_for_spawning(player, chunk, SPAWN_DISTANCE_BLOCK_SQR)
                })
                .map(|(index, _)| index)
                .collect()
        })
    }

    pub fn add_mob(&mut self, chunk: ChunkPos, category: MobCategory) {
        let near = self.players_near(chunk).to_vec();
        for index in near {
            let counts = self.counts[index].get_or_insert([0; CATEGORY_COUNT]);
            counts[category as usize] += 1;
        }
    }

    pub fn can_spawn(&mut self, category: MobCategory, chunk: ChunkPos) -> bool {
        let max = category.max_instances_per_chunk().map_or(-1, i32::from);
        let near = self.players_near(chunk).to_vec();
        near.into_iter()
            .any(|index| self.counts[index].is_none_or(|counts| counts[category as usize] < max))
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SpawnState {
    spawnable_chunks: i32,
    counts: [i32; CATEGORY_COUNT],
    potential: PotentialCalculator,
    local: LocalMobCaps,
    last_checked: Option<(BlockPos, EntityKind)>,
    last_charge: f64,
}

impl SpawnState {
    pub fn new(spawnable_chunks: i32, players: Vec<SpawnPlayer>) -> Self {
        Self {
            spawnable_chunks,
            counts: [0; CATEGORY_COUNT],
            potential: PotentialCalculator::default(),
            local: LocalMobCaps::new(players),
            last_checked: None,
            last_charge: 0.0,
        }
    }

    pub fn spawnable_chunks(&self) -> i32 {
        self.spawnable_chunks
    }

    pub fn count(&self, category: MobCategory) -> i32 {
        self.counts[category as usize]
    }

    pub fn add_existing(
        &mut self,
        kind: EntityKind,
        pos: BlockPos,
        charge: Option<f64>,
        mob: bool,
    ) {
        let category = kind.category();
        if category == MobCategory::Misc {
            return;
        }
        if let Some(charge) = charge {
            self.potential.add_charge(pos, charge);
        }
        if mob {
            self.local.add_mob(pos.chunk(), category);
        }
        self.counts[category as usize] += 1;
    }

    pub fn can_spawn_for_category_global(&self, category: MobCategory, limit: i32) -> bool {
        let cap = limit * self.spawnable_chunks / MAGIC_NUMBER;
        self.counts[category as usize] < cap
    }

    pub fn can_spawn_for_category_local(&mut self, category: MobCategory, chunk: ChunkPos) -> bool {
        self.local.can_spawn(category, chunk)
    }

    pub fn can_spawn(&mut self, kind: EntityKind, pos: BlockPos, cost: Option<(f64, f64)>) -> bool {
        self.last_checked = Some((pos, kind));
        let Some((energy_budget, charge)) = cost else {
            self.last_charge = 0.0;
            return true;
        };
        self.last_charge = charge;
        self.potential.potential_energy_change(pos, charge) <= energy_budget
    }

    pub fn after_spawn(&mut self, kind: EntityKind, pos: BlockPos, charge: impl FnOnce() -> f64) {
        let charge = if self.last_checked == Some((pos, kind)) {
            self.last_charge
        } else {
            charge()
        };
        self.potential.add_charge(pos, charge);
        let category = kind.category();
        self.counts[category as usize] += 1;
        self.local.add_mob(pos.chunk(), category);
    }

    pub fn filtered_categories(
        &self,
        settings: &SpawnSettings,
        spawn_enemies: bool,
        spawn_friendlies: bool,
        game_time: i64,
    ) -> Vec<MobCategory> {
        let mut categories = Vec::with_capacity(SPAWNING_CATEGORIES.len());
        for category in SPAWNING_CATEGORIES {
            let mut spawn_this_tick = true;
            let mut limit = category.max_instances_per_chunk().map_or(-1, i32::from);
            if let Some(configured) = settings.category(category) {
                spawn_this_tick =
                    configured.ticks_per != 0 && game_time % configured.ticks_per == 0;
                limit = configured.limit;
            }
            if !spawn_this_tick || limit == 0 {
                continue;
            }
            if (spawn_enemies || category.is_friendly())
                && (spawn_friendlies || !category.is_persistent())
                && self.can_spawn_for_category_global(category, limit)
            {
                categories.push(category);
            }
        }
        categories
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn player(x: f64, z: f64) -> SpawnPlayer {
        SpawnPlayer {
            position: Vec3::new(x, 64.0, z),
            spectator: false,
        }
    }

    #[test]
    fn global_cap_scales_with_spawnable_chunks() {
        let mut state = SpawnState::new(289, vec![player(0.0, 0.0)]);
        for _ in 0..69 {
            state.add_existing(EntityKind::Zombie, BlockPos::new(0, 64, 0), None, true);
        }
        assert!(state.can_spawn_for_category_global(MobCategory::Monster, 70));
        state.add_existing(EntityKind::Zombie, BlockPos::new(0, 64, 0), None, true);
        assert!(!state.can_spawn_for_category_global(MobCategory::Monster, 70));
        let doubled = SpawnState::new(578, vec![player(0.0, 0.0)]);
        assert!(doubled.can_spawn_for_category_global(MobCategory::Monster, 70));
        assert!(
            !SpawnState::new(289, Vec::new())
                .can_spawn_for_category_global(MobCategory::Monster, -1)
        );
    }

    #[test]
    fn local_cap_counts_mobs_near_each_player() {
        let mut state = SpawnState::new(289, vec![player(0.0, 0.0), player(4000.0, 0.0)]);
        let near = ChunkPos::new(1, 1);
        let far = ChunkPos::new(250, 0);
        for _ in 0..10 {
            state.add_existing(EntityKind::Cow, BlockPos::new(20, 64, 20), None, true);
        }
        assert!(!state.can_spawn_for_category_local(MobCategory::Creature, near));
        assert!(state.can_spawn_for_category_local(MobCategory::Creature, far));
        assert!(state.can_spawn_for_category_local(MobCategory::Monster, near));
        assert!(!state.can_spawn_for_category_local(MobCategory::Monster, ChunkPos::new(40, 40)));
    }

    #[test]
    fn friendly_and_persistent_categories_are_filtered() {
        let state = SpawnState::new(289, vec![player(0.0, 0.0)]);
        let settings = SpawnSettings::default();
        let all = state.filtered_categories(&settings, true, true, 400);
        assert_eq!(all.len(), 7);
        let day = state.filtered_categories(&settings, false, false, 401);
        assert!(!day.contains(&MobCategory::Monster));
        assert!(!day.contains(&MobCategory::Creature));
        assert!(day.contains(&MobCategory::Ambient));
        let mut never = settings;
        never.ambient.ticks_per = 0;
        never.water_ambient.limit = 0;
        let filtered = state.filtered_categories(&never, true, true, 400);
        assert!(!filtered.contains(&MobCategory::Ambient));
        assert!(!filtered.contains(&MobCategory::WaterAmbient));
        assert!(spawn_friendlies(&settings, 800));
        assert!(!spawn_friendlies(&settings, 801));
    }

    #[test]
    fn potential_energy_limits_crowded_spawns() {
        let mut state = SpawnState::new(289, vec![player(0.0, 0.0)]);
        let pos = BlockPos::new(0, 64, 0);
        assert!(state.can_spawn(EntityKind::Skeleton, pos, Some((0.15, 0.7))));
        state.after_spawn(EntityKind::Skeleton, pos, || unreachable!());
        assert!(!state.can_spawn(
            EntityKind::Skeleton,
            BlockPos::new(3, 64, 0),
            Some((0.15, 0.7))
        ));
        assert!(state.can_spawn(
            EntityKind::Skeleton,
            BlockPos::new(30, 64, 0),
            Some((0.15, 0.7))
        ));
        assert!(state.can_spawn(EntityKind::Skeleton, BlockPos::new(3, 64, 0), None));
        assert_eq!(state.count(MobCategory::Monster), 1);
    }

    #[test]
    fn reduced_range_uses_the_inscribed_square_then_the_spigot_radius() {
        let players = [player(10.0, 8.0)];
        assert!(any_player_close_for_spawning(
            &players,
            ChunkPos::new(5, 5),
            Some(6)
        ));
        assert!(!any_player_close_for_spawning(
            &players,
            ChunkPos::new(6, 6),
            Some(6)
        ));
        assert!(any_player_close_for_spawning(
            &players,
            ChunkPos::new(6, 0),
            Some(6)
        ));
        assert!(!any_player_close_for_spawning(
            &players,
            ChunkPos::new(9, 0),
            Some(8)
        ));
        assert!(any_player_close_for_spawning(
            &players,
            ChunkPos::new(7, 0),
            None
        ));
        assert!(!any_player_close_for_spawning(
            &players,
            ChunkPos::new(8, 2),
            None
        ));
        assert_eq!(effective_mob_spawn_range(6, 10), 6);
        assert_eq!(effective_mob_spawn_range(12, 10), 8);
        assert_eq!(effective_mob_spawn_range(6, 4), 4);
    }
}
