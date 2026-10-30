use rspigot_domain::{BlockPos, Vec3};

use crate::classify::is_cauldron;
use crate::view::{floor_level, is_air, is_water};
use crate::{BlockView, MobProfile, MobState, Path, PathType};

const STUCK_CHECK_INTERVAL: i32 = 100;
const STUCK_THRESHOLD_DISTANCE_FACTOR: f32 = 0.25;
const MAX_VERTICAL_DISTANCE_TO_WAYPOINT: f64 = 1.0;
const SURFACE_SCAN: i32 = 16;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct MobTick {
    pub state: MobState,
    pub in_liquid: bool,
    pub passenger: bool,
    pub speed: f32,
    pub game_time: i64,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct MoveTarget {
    pub position: Vec3,
    pub speed_modifier: f64,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Navigation {
    path: Option<Path>,
    speed_modifier: f64,
    tick: i32,
    last_stuck_check: i32,
    last_stuck_check_pos: Vec3,
    timeout_cached_node: BlockPos,
    timeout_timer: i64,
    last_timeout_check: i64,
    timeout_limit: f64,
    max_distance_to_waypoint: f32,
    stuck: bool,
}

impl Default for Navigation {
    fn default() -> Self {
        Self::new()
    }
}

pub fn waypoint_distance(width: f32) -> f32 {
    if width > 0.75 {
        width / 2.0
    } else {
        0.75 - width / 2.0
    }
}

pub fn can_cut_corner(kind: PathType) -> bool {
    !matches!(
        kind,
        PathType::FireInNeighbor | PathType::DamagingInNeighbor | PathType::WalkableDoor
    )
}

impl Navigation {
    pub fn new() -> Self {
        Self {
            path: None,
            speed_modifier: 0.0,
            tick: 0,
            last_stuck_check: 0,
            last_stuck_check_pos: Vec3::ZERO,
            timeout_cached_node: BlockPos::new(0, 0, 0),
            timeout_timer: 0,
            last_timeout_check: 0,
            timeout_limit: 0.0,
            max_distance_to_waypoint: 0.5,
            stuck: false,
        }
    }

    pub fn path(&self) -> Option<&Path> {
        self.path.as_ref()
    }

    pub fn is_done(&self) -> bool {
        self.path.as_ref().is_none_or(Path::is_done)
    }

    pub fn is_in_progress(&self) -> bool {
        !self.is_done()
    }

    pub fn stop(&mut self) {
        self.path = None;
    }

    pub fn is_stuck(&self) -> bool {
        self.stuck
    }

    pub fn max_distance_to_waypoint(&self) -> f32 {
        self.max_distance_to_waypoint
    }

    pub fn speed_modifier(&self) -> f64 {
        self.speed_modifier
    }

    pub fn set_speed_modifier(&mut self, speed_modifier: f64) {
        self.speed_modifier = speed_modifier;
    }

    pub fn reset_stuck_timeout(&mut self) {
        self.timeout_cached_node = BlockPos::new(0, 0, 0);
        self.timeout_timer = 0;
        self.timeout_limit = 0.0;
        self.stuck = false;
    }

    pub fn move_to(
        &mut self,
        path: Option<Path>,
        speed_modifier: f64,
        view: &impl BlockView,
        profile: &MobProfile,
        mob: &MobTick,
    ) -> bool {
        let Some(path) = path else {
            self.path = None;
            return false;
        };
        if !path.same_as(self.path.as_ref()) {
            self.path = Some(path);
        }
        if self.is_done() {
            return false;
        }
        self.trim(view);
        if self.path.as_ref().is_none_or(|path| path.node_count() == 0) {
            return false;
        }
        self.speed_modifier = speed_modifier;
        self.last_stuck_check = self.tick;
        self.last_stuck_check_pos = temp_mob_pos(view, profile, &mob.state);
        true
    }

    fn trim(&mut self, view: &impl BlockView) {
        let Some(path) = self.path.as_mut() else {
            return;
        };
        for index in 0..path.node_count() {
            let node = *path.node(index);
            if !view.state(node.pos).is_some_and(is_cauldron) {
                continue;
            }
            let raised = node.pos.offset(0, 1, 0);
            path.replace_node(index, node.moved(raised));
            if let Some(next) = path.nodes().get(index + 1).copied()
                && node.pos.y >= next.pos.y
            {
                let lifted = BlockPos::new(next.pos.x, raised.y, next.pos.z);
                path.replace_node(index + 1, node.moved(lifted));
            }
        }
    }

    pub fn tick(
        &mut self,
        view: &impl BlockView,
        profile: &MobProfile,
        mob: &MobTick,
    ) -> Option<MoveTarget> {
        self.tick += 1;
        if self.is_done() {
            return None;
        }
        if mob.state.on_ground || mob.in_liquid || mob.passenger {
            self.follow(view, profile, mob);
        } else if let Some(path) = self.path.as_mut().filter(|path| !path.is_done()) {
            let temp = temp_mob_pos(view, profile, &mob.state);
            let next = path.next_entity_pos(profile.width);
            if temp.y > next.y
                && !mob.state.on_ground
                && temp.x.floor() as i32 == next.x.floor() as i32
                && temp.z.floor() as i32 == next.z.floor() as i32
            {
                path.advance();
            }
        }
        if self.is_done() {
            return None;
        }
        let next = self.path.as_ref()?.next_entity_pos(profile.width);
        Some(MoveTarget {
            position: Vec3::new(next.x, ground_y(view, next), next.z),
            speed_modifier: self.speed_modifier,
        })
    }

    fn follow(&mut self, view: &impl BlockView, profile: &MobProfile, mob: &MobTick) {
        let temp = temp_mob_pos(view, profile, &mob.state);
        self.max_distance_to_waypoint = waypoint_distance(profile.width);
        let Some(path) = self.path.as_mut() else {
            return;
        };
        let next = path.next_node_pos();
        let position = mob.state.position;
        let reach = f64::from(self.max_distance_to_waypoint);
        let reached = (position.x - (f64::from(next.x) + 0.5)).abs() < reach
            && (position.z - (f64::from(next.z) + 0.5)).abs() < reach
            && (position.y - f64::from(next.y)).abs() < MAX_VERTICAL_DISTANCE_TO_WAYPOINT;
        if reached || (can_cut_corner(path.next_node().path_type) && aims_past(path, temp)) {
            path.advance();
        }
        self.stuck_detection(temp, mob);
    }

    fn stuck_detection(&mut self, temp: Vec3, mob: &MobTick) {
        if self.tick - self.last_stuck_check > STUCK_CHECK_INTERVAL {
            let speed = if mob.speed >= 1.0 {
                mob.speed
            } else {
                mob.speed * mob.speed
            };
            let threshold = speed * 100.0 * STUCK_THRESHOLD_DISTANCE_FACTOR;
            if temp.distance_squared(self.last_stuck_check_pos) < f64::from(threshold * threshold) {
                self.stuck = true;
                self.stop();
            } else {
                self.stuck = false;
            }
            self.last_stuck_check = self.tick;
            self.last_stuck_check_pos = temp;
        }
        let Some(next) = self
            .path
            .as_ref()
            .filter(|path| !path.is_done())
            .map(Path::next_node_pos)
        else {
            return;
        };
        let time = mob.game_time;
        if next == self.timeout_cached_node {
            self.timeout_timer += time - self.last_timeout_check;
        } else {
            self.timeout_cached_node = next;
            let distance = temp.distance_squared(bottom_center(next)).sqrt();
            self.timeout_limit = if mob.speed > 0.0 {
                distance / f64::from(mob.speed) * 20.0
            } else {
                0.0
            };
        }
        if self.timeout_limit > 0.0 && self.timeout_timer as f64 > self.timeout_limit * 3.0 {
            self.reset_stuck_timeout();
            self.stop();
        }
        self.last_timeout_check = time;
    }
}

fn aims_past(path: &Path, temp: Vec3) -> bool {
    let index = path.next_node_index();
    if index + 1 >= path.node_count() {
        return false;
    }
    let next = bottom_center(path.next_node_pos());
    if temp.distance_squared(next) >= 4.0 {
        return false;
    }
    let following = bottom_center(path.node_pos(index + 1));
    let toward_next = next - temp;
    let toward_following = following - temp;
    let next_length = length_squared(toward_next);
    let following_length = length_squared(toward_following);
    if following_length < next_length || next_length < 0.5 {
        dot(normalize(toward_following), normalize(toward_next)) < 0.0
    } else {
        false
    }
}

fn bottom_center(pos: BlockPos) -> Vec3 {
    Vec3::new(
        f64::from(pos.x) + 0.5,
        f64::from(pos.y),
        f64::from(pos.z) + 0.5,
    )
}

fn length_squared(vec: Vec3) -> f64 {
    vec.x * vec.x + vec.y * vec.y + vec.z * vec.z
}

fn dot(left: Vec3, right: Vec3) -> f64 {
    left.x * right.x + left.y * right.y + left.z * right.z
}

fn normalize(vec: Vec3) -> Vec3 {
    let length = length_squared(vec).sqrt();
    if length < f64::from(1.0e-5_f32) {
        Vec3::ZERO
    } else {
        Vec3::new(vec.x / length, vec.y / length, vec.z / length)
    }
}

pub fn temp_mob_pos(view: &impl BlockView, profile: &MobProfile, mob: &MobState) -> Vec3 {
    let position = mob.position;
    Vec3::new(
        position.x,
        f64::from(surface_y(view, profile, mob)),
        position.z,
    )
}

fn surface_y(view: &impl BlockView, profile: &MobProfile, mob: &MobState) -> i32 {
    let position = mob.position;
    if !mob.in_floatable_fluid || !profile.can_float {
        return (position.y + 0.5).floor() as i32;
    }
    let block = mob.block_pos();
    let mut y = block.y;
    let mut scanned = 0;
    while is_water(view.state(BlockPos::new(block.x, y, block.z))) {
        y += 1;
        scanned += 1;
        if scanned > SURFACE_SCAN {
            return block.y;
        }
    }
    y
}

pub fn ground_y(view: &impl BlockView, position: Vec3) -> f64 {
    let pos = position.block();
    if is_air(view.state(pos.offset(0, -1, 0))) {
        position.y
    } else {
        floor_level(view, pos)
    }
}
