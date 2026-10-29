use crate::perlin::Fractal;
use crate::rng::mix;

#[derive(Debug, Clone, Copy)]
pub(crate) struct ColumnShape {
    pub(crate) height: i32,
    pub(crate) continent: f64,
    pub(crate) river: f64,
    pub(crate) mountain: f64,
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct Climate {
    pub(crate) temperature: f64,
    pub(crate) humidity: f64,
    pub(crate) variant: f64,
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct NoiseScales {
    pub(crate) biome: f64,
    pub(crate) amplitude: f64,
}

const COAST: [(f64, f64); 10] = [
    (-1.0, 20.0),
    (-0.6, 32.0),
    (-0.35, 47.0),
    (-0.2, 58.0),
    (-0.1, 62.0),
    (-0.02, 65.0),
    (0.1, 68.0),
    (0.3, 72.0),
    (0.6, 80.0),
    (1.0, 92.0),
];

fn base_height(continent: f64) -> f64 {
    for pair in COAST.windows(2) {
        let (c0, h0) = pair[0];
        let (c1, h1) = pair[1];
        if continent <= c1 {
            let t = ((continent - c0) / (c1 - c0)).clamp(0.0, 1.0);
            return h0 + (h1 - h0) * t;
        }
    }
    COAST[COAST.len() - 1].1
}

pub(crate) fn smooth(from: f64, to: f64, value: f64) -> f64 {
    let t = ((value - from) / (to - from)).clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}

#[derive(Debug, Clone)]
pub(crate) struct Terrain {
    continents: Fractal,
    erosion: Fractal,
    hills: Fractal,
    ridges: Fractal,
    rivers: Fractal,
    temperature: Fractal,
    humidity: Fractal,
    variant: Fractal,
    scales: NoiseScales,
}

impl Terrain {
    pub(crate) fn new(seed: u64, scales: NoiseScales) -> Self {
        let derive = |salt: u64| mix(seed ^ salt.wrapping_mul(0x9E37_79B9_7F4A_7C15));
        Self {
            continents: Fractal::new(derive(1), 4),
            erosion: Fractal::new(derive(2), 3),
            hills: Fractal::new(derive(3), 4),
            ridges: Fractal::new(derive(4), 3),
            rivers: Fractal::new(derive(5), 2),
            temperature: Fractal::new(derive(6), 2),
            humidity: Fractal::new(derive(7), 2),
            variant: Fractal::new(derive(8), 2),
            scales,
        }
    }

    pub(crate) fn shape(&self, x: i32, z: i32, sea_offset: f64) -> ColumnShape {
        let fx = f64::from(x);
        let fz = f64::from(z);
        let s = 1.0 / self.scales.biome;
        let continent =
            (self.continents.at2(fx * 0.0009 * s, fz * 0.0009 * s) * 2.4).clamp(-1.0, 1.0);
        let erosion = (self.erosion.at2(fx * 0.0014 * s, fz * 0.0014 * s) * 2.4).clamp(-1.0, 1.0);
        let rugged = smooth(0.6, -0.8, erosion);
        let land = smooth(-0.4, 0.0, continent);
        let amplitude = (3.0 + 24.0 * rugged) * (0.45 + 0.55 * land) * self.scales.amplitude;
        let detail = (self.hills.at2(fx / 96.0, fz / 96.0) * 2.2).clamp(-1.0, 1.0);
        let mask = smooth(0.1, 0.5, continent) * smooth(0.2, -0.4, erosion);
        let ridge = 1.0
            - (self.ridges.at2(fx * 0.0026 * s, fz * 0.0026 * s) * 3.0)
                .abs()
                .min(1.0);
        let mountain = mask * (18.0 + 150.0 * ridge * ridge.sqrt()) * self.scales.amplitude;
        let river_distance = self.rivers.at2(fx / 420.0, fz / 420.0).abs();
        let river = (1.0 - smooth(0.0, 0.03, river_distance))
            * smooth(-0.2, -0.05, continent)
            * (1.0 - mask);
        let mut height = base_height(continent) + detail * amplitude + mountain;
        if river > 0.0 && height > 58.0 {
            height += (58.0 - height) * river;
        }
        ColumnShape {
            height: (height + sea_offset).floor() as i32,
            continent,
            river,
            mountain,
        }
    }

    pub(crate) fn climate(&self, x: i32, z: i32) -> Climate {
        let fx = f64::from(x);
        let fz = f64::from(z);
        let s = 1.0 / self.scales.biome;
        Climate {
            temperature: (self.temperature.at2(fx * 0.0013 * s, fz * 0.0013 * s) * 2.6)
                .clamp(-1.0, 1.0),
            humidity: (self.humidity.at2(fx * 0.0018 * s, fz * 0.0018 * s) * 2.6).clamp(-1.0, 1.0),
            variant: (self.variant.at2(fx * 0.006, fz * 0.006) * 2.6).clamp(-1.0, 1.0),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn terrain(seed: u64) -> Terrain {
        Terrain::new(
            seed,
            NoiseScales {
                biome: 1.0,
                amplitude: 1.0,
            },
        )
    }

    #[test]
    fn base_height_is_monotonic() {
        let mut previous = f64::MIN;
        for i in 0..=200 {
            let h = base_height(f64::from(i) / 100.0 - 1.0);
            assert!(h >= previous);
            previous = h;
        }
        assert_eq!(base_height(-5.0), 20.0);
        assert_eq!(base_height(5.0), 92.0);
    }

    #[test]
    fn smooth_handles_both_directions() {
        assert_eq!(smooth(0.0, 1.0, -1.0), 0.0);
        assert_eq!(smooth(0.0, 1.0, 2.0), 1.0);
        assert_eq!(smooth(1.0, 0.0, 2.0), 0.0);
        assert_eq!(smooth(1.0, 0.0, -1.0), 1.0);
        assert!((smooth(0.0, 1.0, 0.5) - 0.5).abs() < 1e-12);
    }

    #[test]
    fn shape_is_deterministic_and_seed_dependent() {
        let a = terrain(1);
        let b = terrain(1);
        let c = terrain(2);
        let mut differing = 0;
        for i in -20..20 {
            let (x, z) = (i * 97, i * -53);
            assert_eq!(a.shape(x, z, 0.0).height, b.shape(x, z, 0.0).height);
            if a.shape(x, z, 0.0).height != c.shape(x, z, 0.0).height {
                differing += 1;
            }
        }
        assert!(differing > 20);
    }

    #[test]
    fn terrain_spans_ocean_and_land() {
        let t = terrain(4242);
        let mut below = 0;
        let mut above = 0;
        let mut high = 0;
        for i in 0..80 {
            for j in 0..80 {
                let h = t.shape(i * 64 - 2560, j * 64 - 2560, 0.0).height;
                if h < 60 {
                    below += 1;
                }
                if h > 66 {
                    above += 1;
                }
                if h > 130 {
                    high += 1;
                }
            }
        }
        assert!(below > 800, "{below}");
        assert!(above > 800, "{above}");
        assert!(high > 0, "{high}");
    }

    #[test]
    fn climate_is_bounded() {
        let t = terrain(3);
        for i in 0..200 {
            let c = t.climate(i * 131, i * -77);
            assert!(
                c.temperature.abs() <= 1.0 && c.humidity.abs() <= 1.0 && c.variant.abs() <= 1.0
            );
        }
    }

    #[test]
    fn sea_offset_shifts_heights() {
        let t = terrain(9);
        assert_eq!(t.shape(10, 10, 5.0).height, t.shape(10, 10, 0.0).height + 5);
    }
}
