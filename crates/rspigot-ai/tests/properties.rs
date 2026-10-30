use std::collections::HashMap;

use proptest::prelude::*;
use rspigot_ai::{BlockView, MobProfile, MobState, PathFinder};
use rspigot_blocks::{BlockKind, StateId};
use rspigot_domain::{BlockPos, Vec3};

struct World {
    blocks: HashMap<BlockPos, StateId>,
}

impl BlockView for World {
    fn state(&self, pos: BlockPos) -> Option<StateId> {
        Some(self.blocks.get(&pos).copied().unwrap_or(StateId::AIR))
    }

    fn min_y(&self) -> i32 {
        0
    }
}

const OBSTACLES: [BlockKind; 6] = [
    BlockKind::Stone,
    BlockKind::OakFence,
    BlockKind::Water,
    BlockKind::Lava,
    BlockKind::OakLeaves,
    BlockKind::Fire,
];

fn world(cells: &[(i32, i32, usize, i32)]) -> World {
    let mut blocks = HashMap::new();
    for x in -10..=10 {
        for z in -10..=10 {
            blocks.insert(BlockPos::new(x, 63, z), BlockKind::Stone.default_state());
        }
    }
    for &(x, z, kind, height) in cells {
        if x == 0 && z == 0 {
            continue;
        }
        for y in 0..height {
            blocks.insert(BlockPos::new(x, 64 + y, z), OBSTACLES[kind].default_state());
        }
    }
    World { blocks }
}

fn cells() -> impl Strategy<Value = Vec<(i32, i32, usize, i32)>> {
    prop::collection::vec((-10..=10, -10..=10, 0..OBSTACLES.len(), 1..=3), 0..60)
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(64))]

    #[test]
    fn paths_are_connected_and_bounded(
        cells in cells(),
        tx in -10..=10,
        tz in -10..=10,
        reach in 0..=2,
        wide in any::<bool>(),
    ) {
        let world = world(&cells);
        let profile = if wide { MobProfile::new(1.4, 0.9) } else { MobProfile::new(0.6, 1.95) };
        let finder = PathFinder::new(profile.clone());
        let mob = MobState {
            position: Vec3::new(0.5, 64.0, 0.5),
            on_ground: true,
            in_floatable_fluid: false,
        };
        let target = BlockPos::new(tx, 64, tz);
        let path = finder.find_path(&world, &mob, &[target], reach).unwrap();
        prop_assert!(path.node_count() >= 1);
        let start = path.node(0).pos;
        prop_assert!(start.y == 64 && (-1..=1).contains(&start.x) && (-1..=1).contains(&start.z));
        if !wide {
            prop_assert_eq!(start, BlockPos::new(0, 64, 0));
        }
        prop_assert!(path.visited_nodes() < finder.max_visited_nodes() as usize);
        for pair in path.nodes().windows(2) {
            let (from, to) = (pair[0].pos, pair[1].pos);
            prop_assert!((from.x - to.x).abs() <= 1 && (from.z - to.z).abs() <= 1);
            prop_assert!(from.x != to.x || from.z != to.z);
            prop_assert!(to.y - from.y <= 1);
            prop_assert!(from.y - to.y <= profile.max_fall_distance);
            prop_assert!(profile.malus(pair[1].path_type) >= 0.0);
            prop_assert!(pair[1].cost_malus >= 0.0);
        }
        if let Some(end) = path.end_node().filter(|_| path.can_reach()) {
            let (dx, dy, dz) = (end.pos.x - tx, end.pos.y - 64, end.pos.z - tz);
            prop_assert!(dx * dx + dy * dy + dz * dz <= reach * reach);
        }
        let again = finder.find_path(&world, &mob, &[target], reach).unwrap();
        prop_assert_eq!(again, path);
    }
}
