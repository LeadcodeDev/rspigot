use rspigot_blocks::{Aabb, StateFlags, StateId};
use rspigot_domain::{BlockPos, ChunkPos};

pub trait BlockView {
    fn state(&self, pos: BlockPos) -> Option<StateId>;

    fn min_y(&self) -> i32 {
        -64
    }

    fn max_y(&self) -> i32 {
        319
    }

    fn chunk_loaded(&self, _chunk: ChunkPos) -> bool {
        true
    }
}

static FULL: [Aabb; 1] = [Aabb::new(0.0, 0.0, 0.0, 1.0, 1.0, 1.0)];

fn boxes(state: Option<StateId>) -> &'static [Aabb] {
    match state {
        Some(state) => state.collision_shape(),
        None => &FULL,
    }
}

pub(crate) fn is_water(state: Option<StateId>) -> bool {
    state.is_some_and(|state| state.flags().contains(StateFlags::WATER))
}

pub(crate) fn is_air(state: Option<StateId>) -> bool {
    state.is_some_and(StateId::is_air)
}

pub(crate) fn floor_level(view: &impl BlockView, pos: BlockPos) -> f64 {
    let below = pos.offset(0, -1, 0);
    let top = boxes(view.state(below))
        .iter()
        .map(|shape| shape.max_y)
        .reduce(f64::max)
        .unwrap_or(0.0);
    f64::from(below.y) + top
}

pub(crate) fn collides(view: &impl BlockView, area: &Aabb) -> bool {
    let low = |value: f64| (value - 1.0e-7).floor() as i32 - 1;
    let high = |value: f64| (value + 1.0e-7).floor() as i32 + 1;
    for x in low(area.min_x)..=high(area.max_x) {
        for y in low(area.min_y)..=high(area.max_y) {
            for z in low(area.min_z)..=high(area.max_z) {
                let pos = BlockPos::new(x, y, z);
                let (dx, dy, dz) = (f64::from(x), f64::from(y), f64::from(z));
                let hit = boxes(view.state(pos)).iter().any(|shape| {
                    shape.min_x + dx < area.max_x
                        && shape.max_x + dx > area.min_x
                        && shape.min_y + dy < area.max_y
                        && shape.max_y + dy > area.min_y
                        && shape.min_z + dz < area.max_z
                        && shape.max_z + dz > area.min_z
                });
                if hit {
                    return true;
                }
            }
        }
    }
    false
}

#[cfg(test)]
mod tests {
    use super::*;
    use rspigot_blocks::BlockKind;

    struct One(BlockPos, StateId);

    impl BlockView for One {
        fn state(&self, pos: BlockPos) -> Option<StateId> {
            Some(if pos == self.0 { self.1 } else { StateId::AIR })
        }
    }

    #[test]
    fn floor_level_reads_shape_below() {
        let origin = BlockPos::new(0, 63, 0);
        let above = BlockPos::new(0, 64, 0);
        let stone = One(origin, BlockKind::Stone.default_state());
        assert_eq!(floor_level(&stone, above), 64.0);
        let slab = One(origin, BlockKind::OakSlab.default_state());
        assert_eq!(floor_level(&slab, above), 63.5);
        let fence = One(origin, BlockKind::OakFence.default_state());
        assert_eq!(floor_level(&fence, above), 64.5);
        assert_eq!(floor_level(&stone, BlockPos::new(0, 66, 0)), 65.0);
    }

    #[test]
    fn collision_is_strict_overlap() {
        let stone = One(BlockPos::new(0, 63, 0), BlockKind::Stone.default_state());
        assert!(!collides(
            &stone,
            &Aabb::new(0.2, 64.0, 0.2, 0.8, 65.9, 0.8)
        ));
        assert!(collides(
            &stone,
            &Aabb::new(0.2, 63.99, 0.2, 0.8, 65.9, 0.8)
        ));
        let fence = One(BlockPos::new(0, 63, 0), BlockKind::OakFence.default_state());
        assert!(collides(&fence, &Aabb::new(0.2, 64.2, 0.2, 0.8, 65.9, 0.8)));
    }
}
