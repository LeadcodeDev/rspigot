use super::terrain::{Climate, ColumnShape};
use crate::resources::Biome;

pub(crate) fn classify(shape: &ColumnShape, climate: &Climate, sea_level: i32) -> Biome {
    let temperature = climate.temperature;
    let humidity = climate.humidity;
    let height = shape.height;
    if shape.river > 0.45 {
        return if temperature < -0.45 {
            Biome::FrozenRiver
        } else {
            Biome::River
        };
    }
    if height < sea_level - 1 {
        return ocean(temperature, height < sea_level - 26);
    }
    if height <= sea_level + 2 && shape.continent < 0.08 {
        return if temperature < -0.45 {
            Biome::SnowyBeach
        } else {
            Biome::Beach
        };
    }
    if height >= sea_level + 87 {
        return if temperature < 0.05 {
            Biome::FrozenPeaks
        } else if temperature > 0.4 && humidity < 0.0 {
            Biome::StonyPeaks
        } else {
            Biome::JaggedPeaks
        };
    }
    if height >= sea_level + 49 && shape.mountain > 15.0 {
        return if temperature < -0.1 {
            Biome::SnowySlopes
        } else {
            Biome::WindsweptHills
        };
    }
    land(temperature, humidity, climate.variant)
}

fn ocean(temperature: f64, deep: bool) -> Biome {
    match (temperature, deep) {
        (t, false) if t < -0.55 => Biome::FrozenOcean,
        (t, true) if t < -0.55 => Biome::DeepFrozenOcean,
        (t, false) if t < -0.2 => Biome::ColdOcean,
        (t, true) if t < -0.2 => Biome::DeepColdOcean,
        (t, false) if t < 0.45 => Biome::Ocean,
        (t, true) if t < 0.45 => Biome::DeepOcean,
        (t, false) if t < 0.7 => Biome::LukewarmOcean,
        (_, true) => Biome::DeepLukewarmOcean,
        (_, false) => Biome::WarmOcean,
    }
}

fn land(temperature: f64, humidity: f64, variant: f64) -> Biome {
    if temperature < -0.5 {
        if humidity > -0.1 {
            Biome::SnowyTaiga
        } else {
            Biome::SnowyPlains
        }
    } else if temperature < -0.2 {
        if humidity > -0.2 {
            Biome::Taiga
        } else {
            Biome::Plains
        }
    } else if temperature < 0.4 {
        if humidity > 0.15 {
            if variant > 0.35 {
                Biome::BirchForest
            } else {
                Biome::Forest
            }
        } else if humidity > -0.2 && variant > 0.2 {
            Biome::Forest
        } else {
            Biome::Plains
        }
    } else if humidity < -0.1 {
        Biome::Desert
    } else if humidity > 0.35 {
        Biome::Forest
    } else {
        Biome::Plains
    }
}

pub(crate) fn is_mountain(biome: Biome) -> bool {
    matches!(
        biome,
        Biome::WindsweptHills
            | Biome::SnowySlopes
            | Biome::JaggedPeaks
            | Biome::FrozenPeaks
            | Biome::StonyPeaks
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    fn shape(height: i32) -> ColumnShape {
        ColumnShape {
            height,
            continent: 0.3,
            river: 0.0,
            mountain: 0.0,
        }
    }

    fn climate(temperature: f64, humidity: f64) -> Climate {
        Climate {
            temperature,
            humidity,
            variant: 0.0,
        }
    }

    #[test]
    fn deep_water_is_deep_ocean() {
        assert_eq!(
            classify(&shape(30), &climate(0.0, 0.0), 63),
            Biome::DeepOcean
        );
        assert_eq!(classify(&shape(50), &climate(0.0, 0.0), 63), Biome::Ocean);
        assert_eq!(
            classify(&shape(50), &climate(-0.9, 0.0), 63),
            Biome::FrozenOcean
        );
        assert_eq!(
            classify(&shape(30), &climate(0.9, 0.0), 63),
            Biome::DeepLukewarmOcean
        );
        assert_eq!(
            classify(&shape(50), &climate(0.9, 0.0), 63),
            Biome::WarmOcean
        );
    }

    #[test]
    fn land_follows_temperature() {
        assert_eq!(
            classify(&shape(80), &climate(-0.9, 0.0), 63),
            Biome::SnowyTaiga
        );
        assert_eq!(classify(&shape(80), &climate(0.8, -0.5), 63), Biome::Desert);
        assert_eq!(classify(&shape(80), &climate(0.0, 0.5), 63), Biome::Forest);
        assert_eq!(classify(&shape(80), &climate(0.0, -0.5), 63), Biome::Plains);
    }

    #[test]
    fn coast_and_rivers_override_land() {
        let mut coastal = shape(64);
        coastal.continent = 0.0;
        assert_eq!(classify(&coastal, &climate(0.0, 0.0), 63), Biome::Beach);
        let mut river = shape(60);
        river.river = 0.9;
        assert_eq!(classify(&river, &climate(0.0, 0.0), 63), Biome::River);
        assert_eq!(
            classify(&river, &climate(-0.9, 0.0), 63),
            Biome::FrozenRiver
        );
    }

    #[test]
    fn altitude_selects_mountain_biomes() {
        assert_eq!(
            classify(&shape(180), &climate(-0.5, 0.0), 63),
            Biome::FrozenPeaks
        );
        assert_eq!(
            classify(&shape(180), &climate(0.8, -0.5), 63),
            Biome::StonyPeaks
        );
        assert_eq!(
            classify(&shape(180), &climate(0.2, 0.5), 63),
            Biome::JaggedPeaks
        );
        let mut slope = shape(130);
        slope.mountain = 40.0;
        assert_eq!(
            classify(&slope, &climate(-0.5, 0.0), 63),
            Biome::SnowySlopes
        );
        assert_eq!(
            classify(&slope, &climate(0.2, 0.0), 63),
            Biome::WindsweptHills
        );
        assert!(is_mountain(Biome::JaggedPeaks));
        assert!(!is_mountain(Biome::Plains));
    }
}
