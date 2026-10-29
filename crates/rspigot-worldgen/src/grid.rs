pub(crate) const POINTS: usize = 5;

pub(crate) const WEIGHTS: [f32; 4] = [0.0, 0.25, 0.5, 0.75];

#[derive(Debug)]
pub(crate) struct Grid<const N: usize> {
    levels: usize,
    values: Vec<[f32; N]>,
}

impl<const N: usize> Grid<N> {
    pub(crate) fn build(
        levels: usize,
        mut sample: impl FnMut(usize, usize, usize) -> [f32; N],
    ) -> Self {
        let mut values = Vec::with_capacity(levels * POINTS * POINTS);
        for j in 0..levels {
            for k in 0..POINTS {
                for i in 0..POINTS {
                    values.push(sample(i, j, k));
                }
            }
        }
        Self { levels, values }
    }

    pub(crate) fn cell_levels(&self) -> usize {
        self.levels.saturating_sub(1)
    }

    pub(crate) fn corners(&self, i: usize, j: usize, k: usize) -> [[f32; N]; 8] {
        let at = |dx: usize, dy: usize, dz: usize| {
            self.values[((j + dy) * POINTS + k + dz) * POINTS + i + dx]
        };
        [
            at(0, 0, 0),
            at(1, 0, 0),
            at(0, 1, 0),
            at(1, 1, 0),
            at(0, 0, 1),
            at(1, 0, 1),
            at(0, 1, 1),
            at(1, 1, 1),
        ]
    }
}

pub(crate) fn trilinear<const N: usize>(
    corners: &[[f32; N]; 8],
    tx: f32,
    ty: f32,
    tz: f32,
) -> [f32; N] {
    std::array::from_fn(|n| {
        let x00 = corners[0][n] + (corners[1][n] - corners[0][n]) * tx;
        let x10 = corners[2][n] + (corners[3][n] - corners[2][n]) * tx;
        let x01 = corners[4][n] + (corners[5][n] - corners[4][n]) * tx;
        let x11 = corners[6][n] + (corners[7][n] - corners[6][n]) * tx;
        let y0 = x00 + (x10 - x00) * ty;
        let y1 = x01 + (x11 - x01) * ty;
        y0 + (y1 - y0) * tz
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn trilinear_hits_corners_and_center() {
        let corners: [[f32; 1]; 8] = std::array::from_fn(|i| [i as f32]);
        assert_eq!(trilinear(&corners, 0.0, 0.0, 0.0), [0.0]);
        assert_eq!(trilinear(&corners, 1.0, 0.0, 0.0), [1.0]);
        assert_eq!(trilinear(&corners, 0.0, 1.0, 0.0), [2.0]);
        assert_eq!(trilinear(&corners, 0.0, 0.0, 1.0), [4.0]);
        assert_eq!(trilinear(&corners, 1.0, 1.0, 1.0), [7.0]);
        assert_eq!(trilinear(&corners, 0.5, 0.5, 0.5), [3.5]);
    }

    #[test]
    fn grid_corners_follow_layout() {
        let grid = Grid::<1>::build(3, |i, j, k| [(i + 10 * k + 100 * j) as f32]);
        assert_eq!(grid.cell_levels(), 2);
        let corners = grid.corners(1, 1, 2);
        assert_eq!(corners[0], [1.0 + 20.0 + 100.0]);
        assert_eq!(corners[1], [2.0 + 20.0 + 100.0]);
        assert_eq!(corners[2], [1.0 + 20.0 + 200.0]);
        assert_eq!(corners[4], [1.0 + 30.0 + 100.0]);
        assert_eq!(corners[7], [2.0 + 30.0 + 200.0]);
    }
}
