use std::time::Duration;

use mavlink::dialects::ardupilotmega::GLOBAL_POSITION_INT_DATA;

#[derive(Debug, Clone, Copy)]
pub struct Position {
    pub time_boot: Duration,
    pub lat_deg: f64,
    pub lon_deg: f64,
    pub alt_msl_m: f64,
    pub alt_rel_m: f64,
    pub vel_north_m_s: f64,
    pub vel_east_m_s: f64,
    pub vel_up_m_s: f64,
    pub heading_deg: Option<f64>,
}

impl From<GLOBAL_POSITION_INT_DATA> for Position {
    fn from(d: GLOBAL_POSITION_INT_DATA) -> Self {
        Self {
            time_boot: Duration::from_millis(d.time_boot_ms as u64),
            lat_deg: d.lat as f64 / 1e7,
            lon_deg: d.lon as f64 / 1e7,
            alt_msl_m: d.alt as f64 / 1000.0,
            alt_rel_m: d.relative_alt as f64 / 1000.0,
            vel_north_m_s: d.vx as f64 / 100.0,
            vel_east_m_s: d.vy as f64 / 100.0,
            vel_up_m_s: -(d.vz as f64) / 100.0,
            heading_deg: if d.hdg == u16::MAX {
                None
            } else {
                Some(d.hdg as f64 / 100.0)
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn position_converts_lat_lon_correctly() {
        let raw = GLOBAL_POSITION_INT_DATA {
            time_boot_ms: 0,
            lat: 473377000,      // 47.3377000°
            lon: 85432100,       //  8.5432100°
            alt: 584000,         // 584.0 m MSL
            relative_alt: 10000, // 10.0 m AGL
            vx: 200,
            vy: -150,
            vz: -50,   // 2.0 m/s N, -1.5 m/s E, +0.5 m/s up
            hdg: 9000, // 90.0°
        };
        let pos: Position = raw.into();
        assert!((pos.lat_deg - 47.3377).abs() < 1e-6);
        assert!((pos.alt_rel_m - 10.0).abs() < 1e-6);
        assert!((pos.vel_up_m_s - 0.5).abs() < 1e-6);
        assert_eq!(pos.heading_deg, Some(90.0));
    }
}
