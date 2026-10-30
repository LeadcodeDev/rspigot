use rspigot_domain::BlockPos;

pub fn manhattan_ordered(
    origin: BlockPos,
    reach_x: i32,
    reach_y: i32,
    reach_z: i32,
) -> impl Iterator<Item = BlockPos> {
    let max_depth = reach_x + reach_y + reach_z;
    let mut depth = 0;
    let mut max_x = 0;
    let mut max_y = 0;
    let mut x = 0;
    let mut y = 0;
    let mut mirror: Option<BlockPos> = None;
    let mut done = false;
    std::iter::from_fn(move || {
        if let Some(pos) = mirror.take() {
            return Some(pos);
        }
        if done {
            return None;
        }
        loop {
            if y > max_y {
                x += 1;
                if x > max_x {
                    depth += 1;
                    if depth > max_depth {
                        done = true;
                        return None;
                    }
                    max_x = reach_x.min(depth);
                    x = -max_x;
                }
                max_y = reach_y.min(depth - x.abs());
                y = -max_y;
            }
            let (dx, dy) = (x, y);
            let dz = depth - dx.abs() - dy.abs();
            y += 1;
            if dz <= reach_z {
                if dz != 0 {
                    mirror = Some(BlockPos::new(origin.x + dx, origin.y + dy, origin.z - dz));
                }
                return Some(BlockPos::new(origin.x + dx, origin.y + dy, origin.z + dz));
            }
        }
    })
}

pub fn closer_to_center_than(pos: BlockPos, point: rspigot_domain::Vec3, distance: f64) -> bool {
    let dx = f64::from(pos.x) + 0.5 - point.x;
    let dy = f64::from(pos.y) + 0.5 - point.y;
    let dz = f64::from(pos.z) + 0.5 - point.z;
    dx * dx + dy * dy + dz * dz < distance * distance
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn manhattan_order_starts_at_the_origin_and_grows_by_distance() {
        let origin = BlockPos::new(10, 64, -3);
        let all: Vec<BlockPos> = manhattan_ordered(origin, 5, 1, 5).collect();
        assert_eq!(all[0], origin);
        assert_eq!(all[1], BlockPos::new(9, 64, -3));
        assert_eq!(all[2], BlockPos::new(10, 63, -3));
        assert_eq!(all[3], BlockPos::new(10, 64, -2));
        assert_eq!(all[4], BlockPos::new(10, 64, -4));
        let distance =
            |p: &BlockPos| (p.x - origin.x).abs() + (p.y - origin.y).abs() + (p.z - origin.z).abs();
        assert!(all.windows(2).all(|w| distance(&w[0]) <= distance(&w[1])));
        let unique: std::collections::HashSet<BlockPos> = all.iter().copied().collect();
        assert_eq!(all.len(), 11 * 3 * 11);
        assert_eq!(unique.len(), all.len());
    }
}
