use super::aabb::Aabb;
use super::physics::BlockView;
use crate::{BlockPos, Vec3, mth};

const CLIP_EPSILON: f64 = 1.0e-7;
const INSIDE_PROBE: f64 = 0.001;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct BlockHit {
    pub pos: BlockPos,
    pub location: Vec3,
    pub inside: bool,
}

#[allow(clippy::too_many_arguments)]
fn clip_point(
    scale: &mut f64,
    da: f64,
    db: f64,
    dc: f64,
    plane: f64,
    b: (f64, f64),
    c: (f64, f64),
    origin: (f64, f64, f64),
) -> bool {
    let t = (plane - origin.0) / da;
    let pb = origin.1 + t * db;
    let pc = origin.2 + t * dc;
    if 0.0 < t
        && t < *scale
        && b.0 - CLIP_EPSILON < pb
        && pb < b.1 + CLIP_EPSILON
        && c.0 - CLIP_EPSILON < pc
        && pc < c.1 + CLIP_EPSILON
    {
        *scale = t;
        return true;
    }
    false
}

fn clip_scale(aabb: &Aabb, from: Vec3, delta: Vec3, scale: &mut f64) -> bool {
    let (dx, dy, dz) = (delta.x, delta.y, delta.z);
    let mut hit = false;
    let x_origin = (from.x, from.y, from.z);
    let yz = ((aabb.min_y, aabb.max_y), (aabb.min_z, aabb.max_z));
    if dx > CLIP_EPSILON {
        hit |= clip_point(scale, dx, dy, dz, aabb.min_x, yz.0, yz.1, x_origin);
    } else if dx < -CLIP_EPSILON {
        hit |= clip_point(scale, dx, dy, dz, aabb.max_x, yz.0, yz.1, x_origin);
    }
    let y_origin = (from.y, from.z, from.x);
    let zx = ((aabb.min_z, aabb.max_z), (aabb.min_x, aabb.max_x));
    if dy > CLIP_EPSILON {
        hit |= clip_point(scale, dy, dz, dx, aabb.min_y, zx.0, zx.1, y_origin);
    } else if dy < -CLIP_EPSILON {
        hit |= clip_point(scale, dy, dz, dx, aabb.max_y, zx.0, zx.1, y_origin);
    }
    let z_origin = (from.z, from.x, from.y);
    let xy = ((aabb.min_x, aabb.max_x), (aabb.min_y, aabb.max_y));
    if dz > CLIP_EPSILON {
        hit |= clip_point(scale, dz, dx, dy, aabb.min_z, xy.0, xy.1, z_origin);
    } else if dz < -CLIP_EPSILON {
        hit |= clip_point(scale, dz, dx, dy, aabb.max_z, xy.0, xy.1, z_origin);
    }
    hit
}

pub fn clip_box(aabb: &Aabb, from: Vec3, to: Vec3) -> Option<Vec3> {
    let delta = to - from;
    let mut scale = 1.0;
    clip_scale(aabb, from, delta, &mut scale).then(|| from + delta.scale(scale))
}

pub fn contains(aabb: &Aabb, point: Vec3) -> bool {
    point.x >= aabb.min_x
        && point.x < aabb.max_x
        && point.y >= aabb.min_y
        && point.y < aabb.max_y
        && point.z >= aabb.min_z
        && point.z < aabb.max_z
}

fn block_boxes(view: &impl BlockView, pos: BlockPos) -> Vec<Aabb> {
    let state = view.block_or_air(pos);
    let origin = Vec3::new(f64::from(pos.x), f64::from(pos.y), f64::from(pos.z));
    state
        .collision_shape()
        .iter()
        .map(|b| Aabb::new(b.min_x, b.min_y, b.min_z, b.max_x, b.max_y, b.max_z).move_by(origin))
        .collect()
}

fn clip_block(view: &impl BlockView, from: Vec3, to: Vec3, pos: BlockPos) -> Option<BlockHit> {
    let boxes = block_boxes(view, pos);
    if boxes.is_empty() {
        return None;
    }
    let delta = to - from;
    if delta.length_sqr() < CLIP_EPSILON {
        return None;
    }
    let probe = from + delta.scale(INSIDE_PROBE);
    if boxes.iter().any(|b| contains(b, probe)) {
        return Some(BlockHit {
            pos,
            location: probe,
            inside: true,
        });
    }
    let mut scale = 1.0;
    let mut hit = false;
    for b in &boxes {
        hit |= clip_scale(b, from, delta, &mut scale);
    }
    hit.then(|| BlockHit {
        pos,
        location: from + delta.scale(scale),
        inside: false,
    })
}

pub fn clip_blocks(view: &impl BlockView, from: Vec3, to: Vec3) -> Option<BlockHit> {
    if from == to {
        return None;
    }
    let end = Vec3::new(
        mth::lerp(-CLIP_EPSILON, to.x, from.x),
        mth::lerp(-CLIP_EPSILON, to.y, from.y),
        mth::lerp(-CLIP_EPSILON, to.z, from.z),
    );
    let start = Vec3::new(
        mth::lerp(-CLIP_EPSILON, from.x, to.x),
        mth::lerp(-CLIP_EPSILON, from.y, to.y),
        mth::lerp(-CLIP_EPSILON, from.z, to.z),
    );
    let mut x = mth::floor(start.x);
    let mut y = mth::floor(start.y);
    let mut z = mth::floor(start.z);
    if let Some(hit) = clip_block(view, from, to, BlockPos::new(x, y, z)) {
        return Some(hit);
    }
    let dx = end.x - start.x;
    let dy = end.y - start.y;
    let dz = end.z - start.z;
    let sx = mth::sign(dx);
    let sy = mth::sign(dy);
    let sz = mth::sign(dz);
    let step = |sign: i32, delta: f64| {
        if sign == 0 {
            f64::MAX
        } else {
            f64::from(sign) / delta
        }
    };
    let tx_step = step(sx, dx);
    let ty_step = step(sy, dy);
    let tz_step = step(sz, dz);
    let first = |step: f64, sign: i32, coordinate: f64| {
        step * if sign > 0 {
            1.0 - mth::frac(coordinate)
        } else {
            mth::frac(coordinate)
        }
    };
    let mut tx = first(tx_step, sx, start.x);
    let mut ty = first(ty_step, sy, start.y);
    let mut tz = first(tz_step, sz, start.z);
    while tx <= 1.0 || ty <= 1.0 || tz <= 1.0 {
        if tx < ty {
            if tx < tz {
                x += sx;
                tx += tx_step;
            } else {
                z += sz;
                tz += tz_step;
            }
        } else if ty < tz {
            y += sy;
            ty += ty_step;
        } else {
            z += sz;
            tz += tz_step;
        }
        if let Some(hit) = clip_block(view, from, to, BlockPos::new(x, y, z)) {
            return Some(hit);
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use rspigot_blocks::BlockKind;

    use super::*;
    use crate::BlockStateId;

    struct Wall(HashMap<BlockPos, BlockStateId>);

    impl BlockView for Wall {
        fn block(&self, pos: BlockPos) -> Option<BlockStateId> {
            Some(self.0.get(&pos).copied().unwrap_or(BlockStateId::AIR))
        }
    }

    #[test]
    fn rays_stop_on_the_first_block_face() {
        let mut blocks = HashMap::new();
        blocks.insert(BlockPos::new(3, 64, 0), BlockKind::Stone.default_state());
        let wall = Wall(blocks);
        let hit = clip_blocks(&wall, Vec3::new(0.5, 64.5, 0.5), Vec3::new(5.5, 64.5, 0.5)).unwrap();
        assert_eq!(hit.pos, BlockPos::new(3, 64, 0));
        assert!((hit.location.x - 3.0).abs() < 1.0e-9);
        assert!(clip_blocks(&wall, Vec3::new(0.5, 66.5, 0.5), Vec3::new(5.5, 66.5, 0.5)).is_none());
    }

    #[test]
    fn entity_boxes_clip_on_entry() {
        let target = Aabb::new(2.0, 0.0, -0.5, 3.0, 2.0, 0.5);
        let hit = clip_box(&target, Vec3::new(0.0, 1.0, 0.0), Vec3::new(4.0, 1.0, 0.0)).unwrap();
        assert_eq!(hit, Vec3::new(2.0, 1.0, 0.0));
        assert!(clip_box(&target, Vec3::new(0.0, 3.0, 0.0), Vec3::new(4.0, 3.0, 0.0)).is_none());
    }
}
