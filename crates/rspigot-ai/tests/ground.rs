use std::collections::HashMap;

use rspigot_ai::{BlockView, MobProfile, MobState, Path, PathFinder, PathType, block_path_type};
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

impl World {
    fn floor(radius: i32) -> Self {
        let mut world = Self {
            blocks: HashMap::new(),
        };
        world.fill(
            (-radius, 63, -radius),
            (radius, 63, radius),
            BlockKind::Stone,
        );
        world
    }

    fn set(&mut self, x: i32, y: i32, z: i32, state: StateId) {
        self.blocks.insert(BlockPos::new(x, y, z), state);
    }

    fn fill(&mut self, from: (i32, i32, i32), to: (i32, i32, i32), kind: BlockKind) {
        for x in from.0..=to.0 {
            for y in from.1..=to.1 {
                for z in from.2..=to.2 {
                    self.set(x, y, z, kind.default_state());
                }
            }
        }
    }

    fn clear(&mut self, from: (i32, i32, i32), to: (i32, i32, i32)) {
        for x in from.0..=to.0 {
            for y in from.1..=to.1 {
                for z in from.2..=to.2 {
                    self.blocks.remove(&BlockPos::new(x, y, z));
                }
            }
        }
    }

    fn door(&mut self, kind: BlockKind, x: i32, z: i32, open: bool) {
        let open = if open { "true" } else { "false" };
        let lower = kind.state_with(&[("half", "lower"), ("open", open)]);
        let upper = kind.state_with(&[("half", "upper"), ("open", open)]);
        self.set(x, 64, z, lower.unwrap());
        self.set(x, 65, z, upper.unwrap());
    }
}

fn zombie() -> MobProfile {
    let mut profile = MobProfile::new(0.6, 1.95);
    profile.follow_range = 35.0;
    profile
}

fn standing(x: f64, y: f64, z: f64) -> MobState {
    MobState {
        position: Vec3::new(x, y, z),
        on_ground: true,
        in_floatable_fluid: false,
    }
}

fn search(world: &World, profile: MobProfile, target: BlockPos, reach: i32) -> Path {
    PathFinder::new(profile)
        .find_path(world, &standing(0.5, 64.0, 0.5), &[target], reach)
        .unwrap()
}

fn positions(path: &Path) -> Vec<BlockPos> {
    (0..path.node_count()).map(|i| path.node(i).pos).collect()
}

fn adjacent(path: &Path) -> bool {
    positions(path)
        .windows(2)
        .all(|pair| (pair[0].x - pair[1].x).abs() <= 1 && (pair[0].z - pair[1].z).abs() <= 1)
}

#[test]
fn flat_floor_gives_straight_path() {
    let world = World::floor(20);
    let path = search(&world, zombie(), BlockPos::new(10, 64, 0), 0);
    assert!(path.can_reach());
    assert_eq!(path.node_count(), 11);
    let expected: Vec<_> = (0..=10).map(|x| BlockPos::new(x, 64, 0)).collect();
    assert_eq!(positions(&path), expected);
    assert_eq!(path.node(0).path_type, PathType::Walkable);
    assert_eq!(path.target(), BlockPos::new(10, 64, 0));
    assert_eq!(path.dist_to_target(), 0.0);
}

#[test]
fn reach_range_stops_one_node_early() {
    let world = World::floor(20);
    let path = search(&world, zombie(), BlockPos::new(10, 64, 0), 1);
    assert!(path.can_reach());
    assert_eq!(path.node_count(), 10);
    assert_eq!(
        path.end_node().map(|node| node.pos),
        Some(BlockPos::new(9, 64, 0))
    );
    assert_eq!(path.dist_to_target(), 1.0);
}

#[test]
fn wall_forces_detour() {
    let mut world = World::floor(20);
    world.fill((5, 64, -3), (5, 66, 3), BlockKind::Stone);
    let path = search(&world, zombie(), BlockPos::new(10, 64, 0), 0);
    assert!(path.can_reach());
    assert!(adjacent(&path));
    assert!(
        positions(&path)
            .iter()
            .all(|pos| !(pos.x == 5 && pos.z.abs() <= 3))
    );
    assert!(positions(&path).iter().any(|pos| pos.z.abs() >= 4));
}

#[test]
fn one_block_step_is_climbed() {
    let mut world = World::floor(20);
    world.fill((5, 64, -20), (20, 64, 20), BlockKind::Stone);
    let path = search(&world, zombie(), BlockPos::new(8, 65, 0), 0);
    assert!(path.can_reach());
    let pos = positions(&path);
    assert!(pos.contains(&BlockPos::new(4, 64, 0)));
    assert!(pos.contains(&BlockPos::new(5, 65, 0)));
}

#[test]
fn two_block_step_is_not_climbed() {
    let mut world = World::floor(20);
    world.fill((5, 64, -20), (20, 65, 20), BlockKind::Stone);
    let path = search(&world, zombie(), BlockPos::new(8, 66, 0), 0);
    assert!(!path.can_reach());
    assert!(positions(&path).iter().all(|pos| pos.y == 64));
}

#[test]
fn three_block_drop_is_taken() {
    let mut world = World::floor(20);
    world.clear((5, 63, -20), (20, 63, 20));
    world.fill((5, 60, -20), (20, 60, 20), BlockKind::Stone);
    let path = search(&world, zombie(), BlockPos::new(8, 61, 0), 0);
    assert!(path.can_reach());
    assert!(positions(&path).contains(&BlockPos::new(5, 61, 0)));
}

#[test]
fn four_block_drop_is_refused_without_alternative() {
    let mut world = World::floor(20);
    world.clear((5, 63, -20), (20, 63, 20));
    world.fill((5, 59, -20), (20, 59, 20), BlockKind::Stone);
    let path = search(&world, zombie(), BlockPos::new(8, 60, 0), 0);
    assert!(!path.can_reach());
    assert!(positions(&path).iter().all(|pos| pos.y == 64));
}

#[test]
fn four_block_drop_is_avoided_when_stairs_exist() {
    let mut world = World::floor(20);
    world.clear((5, 63, -20), (20, 63, 20));
    world.fill((5, 59, -20), (20, 59, 20), BlockKind::Stone);
    world.fill((5, 60, 3), (5, 62, 4), BlockKind::Stone);
    world.fill((6, 60, 3), (6, 61, 4), BlockKind::Stone);
    world.fill((7, 60, 3), (7, 60, 4), BlockKind::Stone);
    let path = search(&world, zombie(), BlockPos::new(8, 60, 0), 0);
    assert!(path.can_reach());
    let pos = positions(&path);
    assert!(pos.windows(2).all(|pair| pair[0].y - pair[1].y <= 3));
    assert!(
        pos.iter()
            .any(|pos| (5..=7).contains(&pos.x) && (3..=4).contains(&pos.z))
    );
}

fn zero_malus(profile: MobProfile, kinds: &[PathType]) -> MobProfile {
    kinds
        .iter()
        .fold(profile, |profile, kind| profile.with_malus(*kind, 0.0))
}

#[test]
fn water_cost_steers_around_water() {
    let mut world = World::floor(20);
    world.fill((5, 64, -3), (5, 64, 3), BlockKind::Water);
    let path = search(&world, zombie(), BlockPos::new(10, 64, 0), 0);
    assert!(path.can_reach());
    assert!((0..path.node_count()).all(|i| path.node(i).path_type != PathType::Water));
    assert!(path.node_count() > 11);
    let free = zero_malus(zombie(), &[PathType::Water, PathType::WaterBorder]);
    let path = search(&world, free, BlockPos::new(10, 64, 0), 0);
    assert_eq!(path.node_count(), 11);
    assert_eq!(path.node(5).path_type, PathType::Water);
    assert_eq!(path.node(4).path_type, PathType::WaterBorder);
}

#[test]
fn lava_blocks_and_neighbours_cost() {
    let mut world = World::floor(20);
    world.fill((5, 64, -3), (5, 64, 3), BlockKind::Lava);
    let path = search(&world, zombie(), BlockPos::new(10, 64, 0), 0);
    assert!(path.can_reach());
    assert!(
        positions(&path)
            .iter()
            .all(|pos| !(pos.x == 5 && pos.z.abs() <= 3))
    );
    world.fill((5, 64, -20), (5, 64, 20), BlockKind::Lava);
    let path = search(&world, zombie(), BlockPos::new(10, 64, 0), 0);
    assert!(!path.can_reach());
}

#[test]
fn fire_cost_steers_around_fire() {
    let mut world = World::floor(20);
    world.set(5, 64, 0, BlockKind::Fire.default_state());
    let path = search(&world, zombie(), BlockPos::new(10, 64, 0), 0);
    assert!(path.can_reach());
    assert!((0..path.node_count()).all(|i| path.node(i).path_type != PathType::Fire));
    let free = zero_malus(zombie(), &[PathType::Fire, PathType::FireInNeighbor]);
    let path = search(&world, free, BlockPos::new(10, 64, 0), 0);
    assert_eq!(path.node_count(), 11);
    assert_eq!(path.node(5).path_type, PathType::Fire);
    assert_eq!(path.node(5).cost_malus, 0.0);
}

fn walled(door: BlockKind) -> World {
    let mut world = World::floor(20);
    world.fill((5, 64, -20), (5, 66, 20), BlockKind::Stone);
    world.door(door, 5, 0, false);
    world
}

#[test]
fn closed_wooden_door_needs_can_open_doors() {
    let world = walled(BlockKind::OakDoor);
    let path = search(&world, zombie(), BlockPos::new(10, 64, 0), 0);
    assert!(!path.can_reach());
    let mut opener = zombie();
    opener.can_open_doors = true;
    let path = search(&world, opener, BlockPos::new(10, 64, 0), 0);
    assert!(path.can_reach());
    let door = positions(&path)
        .iter()
        .position(|pos| *pos == BlockPos::new(5, 64, 0))
        .unwrap();
    assert_eq!(path.node(door).path_type, PathType::WalkableDoor);
}

#[test]
fn iron_door_always_blocks() {
    let world = walled(BlockKind::IronDoor);
    let mut opener = zombie();
    opener.can_open_doors = true;
    let path = search(&world, opener, BlockPos::new(10, 64, 0), 0);
    assert!(!path.can_reach());
}

#[test]
fn open_door_needs_can_pass_doors() {
    let mut world = walled(BlockKind::OakDoor);
    world.door(BlockKind::OakDoor, 5, 0, true);
    let path = search(&world, zombie(), BlockPos::new(10, 64, 0), 0);
    assert!(path.can_reach());
    let mut stay = zombie();
    stay.can_pass_doors = false;
    let path = search(&world, stay, BlockPos::new(10, 64, 0), 0);
    assert!(!path.can_reach());
}

#[test]
fn fence_line_blocks() {
    let mut world = World::floor(20);
    world.fill((5, 64, -20), (5, 64, 20), BlockKind::OakFence);
    let path = search(&world, zombie(), BlockPos::new(10, 64, 0), 0);
    assert!(!path.can_reach());
    assert_eq!(
        block_path_type(&world, BlockPos::new(5, 64, 0)),
        PathType::Fence
    );
    let mut stone = World::floor(20);
    stone.fill((5, 64, -20), (5, 64, 20), BlockKind::Stone);
    let path = search(&stone, zombie(), BlockPos::new(10, 64, 0), 0);
    assert!(path.can_reach());
}

#[test]
fn leaves_are_not_walked_through() {
    let mut world = World::floor(20);
    world.fill((5, 64, -20), (5, 65, 20), BlockKind::OakLeaves);
    assert_eq!(
        block_path_type(&world, BlockPos::new(5, 64, 0)),
        PathType::Leaves
    );
    let path = search(&world, zombie(), BlockPos::new(10, 64, 0), 0);
    assert!(!path.can_reach());
    let mut low = World::floor(20);
    low.fill((5, 64, -20), (5, 64, 20), BlockKind::OakLeaves);
    let path = search(&low, zombie(), BlockPos::new(10, 64, 0), 0);
    assert!(path.can_reach());
    assert!(
        positions(&path)
            .iter()
            .filter(|pos| pos.x == 5)
            .all(|pos| pos.y == 65)
    );
}

#[test]
fn unreachable_target_gives_partial_path_to_closest_node() {
    let mut world = World::floor(20);
    world.clear((5, 63, -20), (20, 63, 20));
    let path = search(&world, zombie(), BlockPos::new(10, 64, 0), 0);
    assert!(!path.can_reach());
    assert_eq!(
        path.end_node().map(|node| node.pos),
        Some(BlockPos::new(4, 64, 0))
    );
    assert_eq!(path.dist_to_target(), 6.0);
}

#[test]
fn visited_nodes_capped_by_follow_range() {
    let world = World::floor(48);
    let target = [BlockPos::new(100, 64, 0)];
    let mob = standing(0.5, 64.0, 0.5);
    let mut finder = PathFinder::new(MobProfile::new(0.6, 1.95));
    assert_eq!(finder.max_visited_nodes(), 256);
    assert_eq!(finder.max_path_length(), 16.0);
    let path = finder.find_path(&world, &mob, &target, 0).unwrap();
    assert!(!path.can_reach());
    assert!(path.visited_nodes() < 255);
    assert!(path.nodes().iter().all(|node| node.walked_distance < 16.0));
    finder.set_max_visited_nodes(20);
    let path = finder.find_path(&world, &mob, &target, 0).unwrap();
    assert_eq!(path.visited_nodes(), 19);
    let finder = PathFinder::new(zombie());
    assert_eq!(finder.max_visited_nodes(), 560);
    assert_eq!(finder.max_path_length(), 35.0);
    let mut short = MobProfile::new(0.6, 1.95);
    short.follow_range = 10.0;
    let finder = PathFinder::new(short);
    assert_eq!(finder.max_visited_nodes(), 160);
    assert_eq!(finder.max_path_length(), 16.0);
}

fn gap_wall(width: i32) -> World {
    let mut world = World::floor(20);
    world.fill((5, 64, -20), (5, 66, 20), BlockKind::Stone);
    world.clear((5, 64, 0), (5, 66, width - 1));
    world
}

#[test]
fn wide_mobs_need_wider_gaps() {
    let narrow = gap_wall(1);
    let cow = MobProfile::new(0.9, 1.4);
    assert!(search(&narrow, cow, BlockPos::new(10, 64, 0), 0).can_reach());
    let spider = MobProfile::new(1.4, 0.9);
    assert!(!search(&narrow, spider.clone(), BlockPos::new(10, 64, 0), 0).can_reach());
    let wide = gap_wall(2);
    assert!(search(&wide, spider, BlockPos::new(10, 64, 0), 0).can_reach());
}

#[test]
fn entity_pos_centres_on_node_by_width() {
    let world = World::floor(20);
    let path = search(&world, zombie(), BlockPos::new(3, 64, 0), 0);
    assert_eq!(path.entity_pos_at_node(0.6, 1), Vec3::new(1.5, 64.0, 0.5));
    assert_eq!(path.entity_pos_at_node(1.4, 1), Vec3::new(2.0, 64.0, 1.0));
    assert_eq!(path.next_entity_pos(0.6), Vec3::new(0.5, 64.0, 0.5));
}

#[test]
fn multiple_targets_pick_reached_shortest() {
    let world = World::floor(20);
    let path = PathFinder::new(zombie())
        .find_path(
            &world,
            &standing(0.5, 64.0, 0.5),
            &[BlockPos::new(8, 64, 0), BlockPos::new(-3, 64, 0)],
            0,
        )
        .unwrap();
    assert!(path.can_reach());
    assert_eq!(path.target(), BlockPos::new(-3, 64, 0));
    assert_eq!(path.node_count(), 4);
}

#[test]
fn no_targets_gives_no_path() {
    let world = World::floor(4);
    assert!(
        PathFinder::new(zombie())
            .find_path(&world, &standing(0.5, 64.0, 0.5), &[], 0)
            .is_none()
    );
}

#[test]
fn unloaded_cells_block() {
    struct Island;
    impl BlockView for Island {
        fn state(&self, pos: BlockPos) -> Option<StateId> {
            if pos.x > 4 {
                None
            } else if pos.y == 63 {
                Some(BlockKind::Stone.default_state())
            } else {
                Some(StateId::AIR)
            }
        }
    }
    let path = PathFinder::new(zombie())
        .find_path(
            &Island,
            &standing(0.5, 64.0, 0.5),
            &[BlockPos::new(10, 64, 0)],
            0,
        )
        .unwrap();
    assert!(!path.can_reach());
    assert!(positions(&path).iter().all(|pos| pos.x <= 4));
}
