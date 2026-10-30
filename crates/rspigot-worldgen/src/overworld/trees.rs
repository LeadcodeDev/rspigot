use crate::buffer::ChunkBuffer;
use crate::resources::{Palette, Tree};
use crate::rng::Rng;

const CONIFER_RADII: [i32; 10] = [0, 1, 2, 1, 2, 3, 2, 3, 3, 3];

pub(crate) fn plant(
    buffer: &mut ChunkBuffer,
    palette: &Palette,
    tree: Tree,
    x: usize,
    z: usize,
    base_y: i32,
    rng: &mut Rng,
) -> bool {
    let trunk = match tree {
        Tree::Oak => 4 + rng.below(3) as i32,
        Tree::Birch => 5 + rng.below(3) as i32,
        Tree::Spruce => 6 + rng.below(4) as i32,
    };
    let top = base_y + trunk - 1;
    if top + 1 > buffer.height().max_y() || !buffer.get(x, base_y, z).is_air() {
        return false;
    }
    let log = palette.log(tree);
    for y in base_y..=top {
        buffer.set(x, y, z, log);
    }
    match tree {
        Tree::Oak | Tree::Birch => broadleaf_crown(buffer, palette, tree, x, z, top, rng),
        Tree::Spruce => conifer_crown(buffer, palette, x, z, base_y, top),
    }
    true
}

fn leaf(
    buffer: &mut ChunkBuffer,
    palette: &Palette,
    tree: Tree,
    (x, z): (usize, usize),
    (dx, dz): (i32, i32),
    y: i32,
    distance: i32,
) {
    let lx = x as i32 + dx;
    let lz = z as i32 + dz;
    if !(0..16).contains(&lx) || !(0..16).contains(&lz) {
        return;
    }
    let (lx, lz) = (lx as usize, lz as usize);
    if buffer.get(lx, y, lz).is_air() {
        buffer.set(lx, y, lz, palette.leaves(tree, distance));
    }
}

fn broadleaf_crown(
    buffer: &mut ChunkBuffer,
    palette: &Palette,
    tree: Tree,
    x: usize,
    z: usize,
    top: i32,
    rng: &mut Rng,
) {
    for dy in -2..=1i32 {
        let radius: i32 = if dy <= -1 { 2 } else { 1 };
        for dz in -radius..=radius {
            for dx in -radius..=radius {
                let corner = dx.abs() == radius && dz.abs() == radius;
                if corner && (dy == 1 || (dy < 0 && rng.below(2) == 0)) {
                    continue;
                }
                if dx == 0 && dz == 0 && dy <= 0 {
                    continue;
                }
                let distance = dx.abs() + dz.abs() + dy.max(0);
                leaf(buffer, palette, tree, (x, z), (dx, dz), top + dy, distance);
            }
        }
    }
}

fn conifer_crown(
    buffer: &mut ChunkBuffer,
    palette: &Palette,
    x: usize,
    z: usize,
    base_y: i32,
    top: i32,
) {
    for (level, y) in (base_y + 2..=top + 1).rev().enumerate() {
        let radius = CONIFER_RADII[level.min(CONIFER_RADII.len() - 1)];
        let reach = radius + i32::from(radius >= 2);
        for dz in -radius..=radius {
            for dx in -radius..=radius {
                if dx.abs() + dz.abs() > reach || (dx == 0 && dz == 0 && y <= top) {
                    continue;
                }
                let distance = dx.abs() + dz.abs() + (y - top).max(0);
                leaf(buffer, palette, Tree::Spruce, (x, z), (dx, dz), y, distance);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rspigot_domain::BlockStateId;
    use rspigot_domain::world::WorldHeight;

    fn grown(tree: Tree, seed: u64) -> (ChunkBuffer, Palette) {
        let palette = Palette::resolve();
        let mut buffer = ChunkBuffer::new(WorldHeight::OVERWORLD);
        let mut rng = Rng::new(seed);
        assert!(plant(&mut buffer, &palette, tree, 8, 8, 70, &mut rng));
        (buffer, palette)
    }

    fn count(buffer: &ChunkBuffer, pred: impl Fn(BlockStateId) -> bool) -> usize {
        buffer
            .raw()
            .iter()
            .filter(|s| pred(BlockStateId(**s)))
            .count()
    }

    fn is_leaf(state: BlockStateId) -> bool {
        state
            .kind()
            .is_some_and(|kind| kind.name().ends_with("_leaves"))
    }

    #[test]
    fn every_tree_has_trunk_and_crown_inside_the_footprint() {
        for tree in [Tree::Oak, Tree::Birch, Tree::Spruce] {
            for seed in 0..20 {
                let (buffer, palette) = grown(tree, seed);
                let logs = count(&buffer, |s| s == palette.log(tree));
                let leaves = count(&buffer, is_leaf);
                assert!((4..=9).contains(&logs), "{tree:?} {logs}");
                assert!(leaves >= 15, "{tree:?} {leaves}");
                for y in 71..80 {
                    assert_eq!(
                        buffer.get(8, y, 8) == palette.log(tree),
                        y < 70 + logs as i32
                    );
                }
            }
        }
    }

    #[test]
    fn crowns_stay_within_three_blocks_of_the_trunk() {
        for tree in [Tree::Oak, Tree::Spruce] {
            let (buffer, _) = grown(tree, 3);
            for y in 60..110 {
                for z in 0..16usize {
                    for x in 0..16usize {
                        if is_leaf(buffer.get(x, y, z)) {
                            assert!(x.abs_diff(8) <= 3 && z.abs_diff(8) <= 3);
                        }
                    }
                }
            }
        }
    }

    #[test]
    fn leaves_never_decay_by_distance() {
        for tree in [Tree::Oak, Tree::Birch, Tree::Spruce] {
            let (buffer, _) = grown(tree, 8);
            for y in 60..110 {
                for z in 0..16usize {
                    for x in 0..16usize {
                        let state = buffer.get(x, y, z);
                        if is_leaf(state) {
                            let distance: i32 =
                                state.property("distance").unwrap().parse().unwrap();
                            assert!((1..=6).contains(&distance));
                            assert_eq!(state.property("persistent"), Some("false"));
                        }
                    }
                }
            }
        }
    }

    #[test]
    fn spruce_is_taller_than_oak() {
        let tallest = |tree| {
            (0..30)
                .map(|seed| {
                    let (buffer, palette) = grown(tree, seed);
                    count(&buffer, |s| s == palette.log(tree))
                })
                .max()
                .unwrap()
        };
        assert!(tallest(Tree::Spruce) > tallest(Tree::Oak));
    }

    #[test]
    fn planting_fails_without_room() {
        let palette = Palette::resolve();
        let mut buffer = ChunkBuffer::new(WorldHeight::OVERWORLD);
        let mut rng = Rng::new(1);
        assert!(!plant(
            &mut buffer,
            &palette,
            Tree::Spruce,
            8,
            8,
            316,
            &mut rng
        ));
        buffer.set(8, 70, 8, palette.log(Tree::Oak));
        assert!(!plant(&mut buffer, &palette, Tree::Oak, 8, 8, 70, &mut rng));
    }

    #[test]
    fn leaves_do_not_replace_solid_blocks() {
        let palette = Palette::resolve();
        let mut buffer = ChunkBuffer::new(WorldHeight::OVERWORLD);
        for y in 60..=72 {
            buffer.fill_layer(y, BlockStateId(1));
        }
        let mut rng = Rng::new(4);
        buffer.set(8, 73, 8, BlockStateId::AIR);
        assert!(plant(&mut buffer, &palette, Tree::Oak, 8, 8, 73, &mut rng));
        assert_eq!(buffer.get(6, 72, 6), BlockStateId(1));
        assert!(count(&buffer, is_leaf) > 0);
    }
}
