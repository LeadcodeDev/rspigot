use rspigot_blocks::Face;

use super::aabb::Aabb;
use super::physics::BlockView;
use crate::{BlockPos, BlockStateId, Vec3, mth};

const CLIP_EPSILON: f64 = 1.0e-7;
const INSIDE_PROBE: f64 = 0.001;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct BlockHit {
    pub pos: BlockPos,
    pub location: Vec3,
    pub inside: bool,
    pub face: Face,
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

fn entry_plane(along: f64, min: f64, max: f64, sides: (Face, Face)) -> Option<(f64, Face)> {
    if along > CLIP_EPSILON {
        Some((min, sides.0))
    } else if along < -CLIP_EPSILON {
        Some((max, sides.1))
    } else {
        None
    }
}

fn clip_scale(aabb: &Aabb, from: Vec3, delta: Vec3, scale: &mut f64) -> Option<Face> {
    let (dx, dy, dz) = (delta.x, delta.y, delta.z);
    let mut face = None;
    let x_origin = (from.x, from.y, from.z);
    let yz = ((aabb.min_y, aabb.max_y), (aabb.min_z, aabb.max_z));
    if let Some((plane, side)) = entry_plane(dx, aabb.min_x, aabb.max_x, (Face::West, Face::East))
        && clip_point(scale, dx, dy, dz, plane, yz.0, yz.1, x_origin)
    {
        face = Some(side);
    }
    let y_origin = (from.y, from.z, from.x);
    let zx = ((aabb.min_z, aabb.max_z), (aabb.min_x, aabb.max_x));
    if let Some((plane, side)) = entry_plane(dy, aabb.min_y, aabb.max_y, (Face::Down, Face::Up))
        && clip_point(scale, dy, dz, dx, plane, zx.0, zx.1, y_origin)
    {
        face = Some(side);
    }
    let z_origin = (from.z, from.x, from.y);
    let xy = ((aabb.min_x, aabb.max_x), (aabb.min_y, aabb.max_y));
    if let Some((plane, side)) = entry_plane(dz, aabb.min_z, aabb.max_z, (Face::North, Face::South))
        && clip_point(scale, dz, dx, dy, plane, xy.0, xy.1, z_origin)
    {
        face = Some(side);
    }
    face
}

pub fn clip_box(aabb: &Aabb, from: Vec3, to: Vec3) -> Option<Vec3> {
    let delta = to - from;
    let mut scale = 1.0;
    clip_scale(aabb, from, delta, &mut scale).map(|_| from + delta.scale(scale))
}

pub fn contains(aabb: &Aabb, point: Vec3) -> bool {
    point.x >= aabb.min_x
        && point.x < aabb.max_x
        && point.y >= aabb.min_y
        && point.y < aabb.max_y
        && point.z >= aabb.min_z
        && point.z < aabb.max_z
}

fn block_boxes(
    view: &impl BlockView,
    pos: BlockPos,
    shape: &impl Fn(BlockStateId) -> &'static [rspigot_blocks::Aabb],
) -> Vec<Aabb> {
    let origin = Vec3::new(f64::from(pos.x), f64::from(pos.y), f64::from(pos.z));
    shape(view.block_or_air(pos))
        .iter()
        .map(|b| Aabb::new(b.min_x, b.min_y, b.min_z, b.max_x, b.max_y, b.max_z).move_by(origin))
        .collect()
}

fn inside_face(delta: Vec3) -> Face {
    let (x, y, z) = (delta.x.abs(), delta.y.abs(), delta.z.abs());
    let (along, sides) = if y >= x && y >= z {
        (delta.y, (Face::Down, Face::Up))
    } else if z >= x {
        (delta.z, (Face::North, Face::South))
    } else {
        (delta.x, (Face::West, Face::East))
    };
    if along > 0.0 { sides.0 } else { sides.1 }
}

fn clip_block(
    view: &impl BlockView,
    from: Vec3,
    to: Vec3,
    pos: BlockPos,
    shape: &impl Fn(BlockStateId) -> &'static [rspigot_blocks::Aabb],
) -> Option<BlockHit> {
    let boxes = block_boxes(view, pos, shape);
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
            face: inside_face(delta),
        });
    }
    let mut scale = 1.0;
    let mut face = None;
    for b in &boxes {
        face = clip_scale(b, from, delta, &mut scale).or(face);
    }
    face.map(|face| BlockHit {
        pos,
        location: from + delta.scale(scale),
        inside: false,
        face,
    })
}

pub fn clip_blocks(view: &impl BlockView, from: Vec3, to: Vec3) -> Option<BlockHit> {
    clip_blocks_with(view, from, to, BlockStateId::collision_shape)
}

pub fn clip_blocks_with(
    view: &impl BlockView,
    from: Vec3,
    to: Vec3,
    shape: impl Fn(BlockStateId) -> &'static [rspigot_blocks::Aabb],
) -> Option<BlockHit> {
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
    if let Some(hit) = clip_block(view, from, to, BlockPos::new(x, y, z), &shape) {
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
        if let Some(hit) = clip_block(view, from, to, BlockPos::new(x, y, z), &shape) {
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
    fn the_face_is_the_side_the_ray_enters() {
        let mut blocks = HashMap::new();
        blocks.insert(BlockPos::new(3, 64, 0), BlockKind::Stone.default_state());
        blocks.insert(BlockPos::new(0, 60, 0), BlockKind::Stone.default_state());
        blocks.insert(BlockPos::new(0, 0, 0), BlockKind::OakSlab.default_state());
        let wall = Wall(blocks);
        let east =
            clip_blocks(&wall, Vec3::new(0.5, 64.5, 0.5), Vec3::new(5.5, 64.5, 0.5)).unwrap();
        assert_eq!((east.pos, east.face), (BlockPos::new(3, 64, 0), Face::West));
        let down =
            clip_blocks(&wall, Vec3::new(0.5, 62.5, 0.5), Vec3::new(0.5, 58.5, 0.5)).unwrap();
        assert_eq!((down.pos, down.face), (BlockPos::new(0, 60, 0), Face::Up));
        let slab = clip_blocks(&wall, Vec3::new(0.5, 2.5, 0.5), Vec3::new(0.5, -1.5, 0.5)).unwrap();
        assert_eq!((slab.pos, slab.face), (BlockPos::new(0, 0, 0), Face::Up));
        assert!((slab.location.y - 0.5).abs() < 1.0e-9);
        let inside =
            clip_blocks(&wall, Vec3::new(3.5, 64.5, 0.5), Vec3::new(5.5, 64.5, 0.5)).unwrap();
        assert!(inside.inside);
        assert_eq!(inside.face, Face::West);
    }

    #[test]
    fn entity_boxes_clip_on_entry() {
        let target = Aabb::new(2.0, 0.0, -0.5, 3.0, 2.0, 0.5);
        let hit = clip_box(&target, Vec3::new(0.0, 1.0, 0.0), Vec3::new(4.0, 1.0, 0.0)).unwrap();
        assert_eq!(hit, Vec3::new(2.0, 1.0, 0.0));
        assert!(clip_box(&target, Vec3::new(0.0, 3.0, 0.0), Vec3::new(4.0, 3.0, 0.0)).is_none());
    }
}
