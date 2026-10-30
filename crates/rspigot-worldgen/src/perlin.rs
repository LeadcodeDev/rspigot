use crate::rng::Rng;

const fn fade(t: f64) -> f64 {
    t * t * t * (t * (t * 6.0 - 15.0) + 10.0)
}

const fn lerp(t: f64, a: f64, b: f64) -> f64 {
    a + t * (b - a)
}

const fn grad2(hash: u8, x: f64, z: f64) -> f64 {
    match hash & 7 {
        0 => x + z,
        1 => -x + z,
        2 => x - z,
        3 => -x - z,
        4 => x,
        5 => -x,
        6 => z,
        _ => -z,
    }
}

const fn grad3(hash: u8, x: f64, y: f64, z: f64) -> f64 {
    let h = hash & 15;
    let u = if h < 8 { x } else { y };
    let v = if h < 4 {
        y
    } else if h == 12 || h == 14 {
        x
    } else {
        z
    };
    (if h & 1 == 0 { u } else { -u }) + (if h & 2 == 0 { v } else { -v })
}

#[derive(Debug, Clone)]
pub(crate) struct Perlin {
    perm: [u8; 512],
    ox: f64,
    oy: f64,
    oz: f64,
}

impl Perlin {
    pub(crate) fn new(rng: &mut Rng) -> Self {
        let mut table: [u8; 256] = std::array::from_fn(|i| i as u8);
        for i in (1..256usize).rev() {
            let j = rng.below(i as u32 + 1) as usize;
            table.swap(i, j);
        }
        let mut perm = [0u8; 512];
        perm[..256].copy_from_slice(&table);
        perm[256..].copy_from_slice(&table);
        Self {
            perm,
            ox: rng.unit() * 256.0,
            oy: rng.unit() * 256.0,
            oz: rng.unit() * 256.0,
        }
    }

    fn p(&self, index: usize) -> usize {
        usize::from(self.perm[index])
    }

    pub(crate) fn noise2(&self, x: f64, z: f64) -> f64 {
        let x = x + self.ox;
        let z = z + self.oz;
        let xf = x.floor();
        let zf = z.floor();
        let xi = (xf as i64 & 255) as usize;
        let zi = (zf as i64 & 255) as usize;
        let fx = x - xf;
        let fz = z - zf;
        let u = fade(fx);
        let v = fade(fz);
        let a = self.p(xi) + zi;
        let b = self.p(xi + 1) + zi;
        lerp(
            v,
            lerp(
                u,
                grad2(self.perm[self.p(a)], fx, fz),
                grad2(self.perm[self.p(b)], fx - 1.0, fz),
            ),
            lerp(
                u,
                grad2(self.perm[self.p(a + 1)], fx, fz - 1.0),
                grad2(self.perm[self.p(b + 1)], fx - 1.0, fz - 1.0),
            ),
        )
    }

    pub(crate) fn noise3(&self, x: f64, y: f64, z: f64) -> f64 {
        let x = x + self.ox;
        let y = y + self.oy;
        let z = z + self.oz;
        let xf = x.floor();
        let yf = y.floor();
        let zf = z.floor();
        let xi = (xf as i64 & 255) as usize;
        let yi = (yf as i64 & 255) as usize;
        let zi = (zf as i64 & 255) as usize;
        let fx = x - xf;
        let fy = y - yf;
        let fz = z - zf;
        let u = fade(fx);
        let v = fade(fy);
        let w = fade(fz);
        let a = self.p(xi) + yi;
        let aa = self.p(a) + zi;
        let ab = self.p(a + 1) + zi;
        let b = self.p(xi + 1) + yi;
        let ba = self.p(b) + zi;
        let bb = self.p(b + 1) + zi;
        lerp(
            w,
            lerp(
                v,
                lerp(
                    u,
                    grad3(self.perm[aa], fx, fy, fz),
                    grad3(self.perm[ba], fx - 1.0, fy, fz),
                ),
                lerp(
                    u,
                    grad3(self.perm[ab], fx, fy - 1.0, fz),
                    grad3(self.perm[bb], fx - 1.0, fy - 1.0, fz),
                ),
            ),
            lerp(
                v,
                lerp(
                    u,
                    grad3(self.perm[aa + 1], fx, fy, fz - 1.0),
                    grad3(self.perm[ba + 1], fx - 1.0, fy, fz - 1.0),
                ),
                lerp(
                    u,
                    grad3(self.perm[ab + 1], fx, fy - 1.0, fz - 1.0),
                    grad3(self.perm[bb + 1], fx - 1.0, fy - 1.0, fz - 1.0),
                ),
            ),
        )
    }
}

#[derive(Debug, Clone)]
pub(crate) struct Fractal {
    layers: Box<[Perlin]>,
    norm: f64,
}

impl Fractal {
    pub(crate) fn new(seed: u64, octaves: usize) -> Self {
        let mut rng = Rng::new(seed);
        let layers: Box<[Perlin]> = (0..octaves).map(|_| Perlin::new(&mut rng)).collect();
        let mut total = 0.0;
        let mut amplitude = 1.0;
        for _ in 0..octaves {
            total += amplitude;
            amplitude *= 0.5;
        }
        Self {
            layers,
            norm: 1.0 / total,
        }
    }

    pub(crate) fn at2(&self, x: f64, z: f64) -> f64 {
        let mut frequency = 1.0;
        let mut amplitude = 1.0;
        let mut sum = 0.0;
        for layer in &*self.layers {
            sum += layer.noise2(x * frequency, z * frequency) * amplitude;
            frequency *= 2.0;
            amplitude *= 0.5;
        }
        sum * self.norm
    }

    pub(crate) fn at3(&self, x: f64, y: f64, z: f64) -> f64 {
        let mut frequency = 1.0;
        let mut amplitude = 1.0;
        let mut sum = 0.0;
        for layer in &*self.layers {
            sum += layer.noise3(x * frequency, y * frequency, z * frequency) * amplitude;
            frequency *= 2.0;
            amplitude *= 0.5;
        }
        sum * self.norm
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn noise_is_deterministic_and_bounded() {
        let a = Perlin::new(&mut Rng::new(5));
        let b = Perlin::new(&mut Rng::new(5));
        for i in 0..500 {
            let x = f64::from(i) * 0.37 - 80.0;
            let z = f64::from(i) * 0.91 + 12.0;
            let y = f64::from(i) * 0.13;
            assert_eq!(a.noise2(x, z), b.noise2(x, z));
            assert_eq!(a.noise3(x, y, z), b.noise3(x, y, z));
            assert!(a.noise2(x, z).abs() <= 1.0);
            assert!(a.noise3(x, y, z).abs() <= 1.0);
        }
    }

    #[test]
    fn noise_is_continuous() {
        let p = Perlin::new(&mut Rng::new(11));
        let mut previous = p.noise2(0.0, 3.3);
        for i in 1..2000 {
            let value = p.noise2(f64::from(i) * 0.01, 3.3);
            assert!((value - previous).abs() < 0.1);
            previous = value;
        }
    }

    #[test]
    fn noise_varies_and_differs_by_seed() {
        let a = Perlin::new(&mut Rng::new(1));
        let b = Perlin::new(&mut Rng::new(2));
        let mut distinct = 0;
        let mut min = f64::MAX;
        let mut max = f64::MIN;
        for i in 0..400 {
            let x = f64::from(i) * 0.53;
            let v = a.noise2(x, x * 0.7);
            min = min.min(v);
            max = max.max(v);
            if (v - b.noise2(x, x * 0.7)).abs() > 1e-9 {
                distinct += 1;
            }
        }
        assert!(max - min > 0.5);
        assert!(distinct > 300);
    }

    #[test]
    fn negative_coordinates_are_supported() {
        let p = Perlin::new(&mut Rng::new(8));
        let value = p.noise3(-1234.56, -70.2, -9999.9);
        assert!(value.is_finite());
        assert!(p.noise2(-0.0001, -0.0001).is_finite());
    }

    #[test]
    fn fractal_is_normalized() {
        let f = Fractal::new(3, 4);
        for i in 0..400 {
            let v = f.at2(f64::from(i) * 0.41, f64::from(i) * 0.29);
            assert!(v.abs() <= 1.0);
            let w = f.at3(
                f64::from(i) * 0.41,
                f64::from(i) * 0.17,
                f64::from(i) * 0.29,
            );
            assert!(w.abs() <= 1.0);
        }
    }
}
