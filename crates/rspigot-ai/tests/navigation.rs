use std::collections::HashMap;

use rspigot_ai::{
    BlockView, MobProfile, MobState, MobTick, Navigation, Path, PathNode, PathType,
    waypoint_distance,
};
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

fn flat() -> World {
    let mut blocks = HashMap::new();
    for x in -8..=8 {
        for z in -8..=8 {
            blocks.insert(BlockPos::new(x, 63, z), BlockKind::Stone.default_state());
        }
    }
    World { blocks }
}

fn line(length: i32) -> Path {
    let nodes = (0..length)
        .map(|x| PathNode::new(BlockPos::new(x, 64, 0), PathType::Walkable))
        .collect();
    Path::new(nodes, BlockPos::new(length - 1, 64, 0), true)
}

fn tick(x: f64, speed: f32, game_time: i64) -> MobTick {
    MobTick {
        state: MobState {
            position: Vec3::new(x, 64.0, 0.5),
            on_ground: true,
            in_floatable_fluid: false,
        },
        in_liquid: false,
        passenger: false,
        speed,
        game_time,
    }
}

#[test]
fn waypoint_distance_follows_width() {
    assert_eq!(waypoint_distance(0.6), 0.45);
    assert_eq!(waypoint_distance(0.75), 0.375);
    assert_eq!(waypoint_distance(1.4), 0.7);
}

#[test]
fn follows_nodes_one_by_one() {
    let world = flat();
    let profile = MobProfile::new(0.6, 1.95);
    let mut navigation = Navigation::new();
    assert!(navigation.move_to(Some(line(4)), 1.0, &world, &profile, &tick(0.5, 0.25, 0)));
    let target = navigation.tick(&world, &profile, &tick(0.5, 0.25, 1));
    assert_eq!(navigation.path().map(Path::next_node_index), Some(1));
    assert_eq!(navigation.max_distance_to_waypoint(), 0.45);
    let target = target.unwrap();
    assert_eq!(target.position, Vec3::new(1.5, 64.0, 0.5));
    assert_eq!(target.speed_modifier, 1.0);
    navigation.tick(&world, &profile, &tick(0.5, 0.25, 2));
    assert_eq!(navigation.path().map(Path::next_node_index), Some(1));
    navigation.tick(&world, &profile, &tick(1.5, 0.25, 3));
    assert_eq!(navigation.path().map(Path::next_node_index), Some(2));
    navigation.tick(&world, &profile, &tick(2.5, 0.25, 4));
    let last = navigation.tick(&world, &profile, &tick(3.5, 0.25, 5));
    assert!(last.is_none());
    assert!(navigation.is_done());
    assert!(!navigation.is_stuck());
}

#[test]
fn standing_still_is_stuck_after_hundred_ticks() {
    let world = flat();
    let profile = MobProfile::new(0.6, 1.95);
    let mut navigation = Navigation::new();
    assert!(navigation.move_to(Some(line(6)), 1.0, &world, &profile, &tick(0.5, 0.25, 0)));
    for time in 1..=100 {
        navigation.tick(&world, &profile, &tick(0.5, 0.25, time));
    }
    assert!(navigation.is_in_progress());
    navigation.tick(&world, &profile, &tick(0.5, 0.25, 101));
    assert!(navigation.is_stuck());
    assert!(navigation.is_done());
}

#[test]
fn slow_progress_times_out() {
    let world = flat();
    let profile = MobProfile::new(0.6, 1.95);
    let mut navigation = Navigation::new();
    assert!(navigation.move_to(Some(line(6)), 1.0, &world, &profile, &tick(0.5, 2.0, 0)));
    for time in 1..=31 {
        navigation.tick(&world, &profile, &tick(0.5, 2.0, time));
    }
    assert!(navigation.is_in_progress());
    navigation.tick(&world, &profile, &tick(0.5, 2.0, 32));
    assert!(navigation.is_done());
    assert!(!navigation.is_stuck());
}

#[test]
fn airborne_mob_above_next_node_advances() {
    let world = flat();
    let profile = MobProfile::new(0.6, 1.95);
    let mut navigation = Navigation::new();
    assert!(navigation.move_to(Some(line(4)), 1.0, &world, &profile, &tick(0.5, 0.25, 0)));
    let mut airborne = tick(0.5, 0.25, 1);
    airborne.state.position.y = 65.2;
    airborne.state.on_ground = false;
    navigation.tick(&world, &profile, &airborne);
    assert_eq!(navigation.path().map(Path::next_node_index), Some(1));
}

#[test]
fn empty_or_missing_path_is_refused() {
    let world = flat();
    let profile = MobProfile::new(0.6, 1.95);
    let mut navigation = Navigation::new();
    assert!(!navigation.move_to(None, 1.0, &world, &profile, &tick(0.5, 0.25, 0)));
    let empty = Path::new(Vec::new(), BlockPos::new(0, 64, 0), false);
    assert!(!navigation.move_to(Some(empty), 1.0, &world, &profile, &tick(0.5, 0.25, 0)));
    assert!(navigation.is_done());
}

#[test]
fn cauldron_nodes_are_raised() {
    let mut world = flat();
    world
        .blocks
        .insert(BlockPos::new(2, 64, 0), BlockKind::Cauldron.default_state());
    let profile = MobProfile::new(0.6, 1.95);
    let mut navigation = Navigation::new();
    assert!(navigation.move_to(Some(line(4)), 1.0, &world, &profile, &tick(0.5, 0.25, 0)));
    let path = navigation.path().unwrap();
    assert_eq!(path.node(2).pos, BlockPos::new(2, 65, 0));
    assert_eq!(path.node(3).pos, BlockPos::new(3, 65, 0));
    assert_eq!(path.node(1).pos, BlockPos::new(1, 64, 0));
}
