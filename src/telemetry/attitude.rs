use std::time::Duration;

use mavlink::dialects::ardupilotmega::ATTITUDE_DATA;

#[derive(Debug, Clone, Copy)]
pub struct Attitude {
    pub time_boot: Duration,
    /// Roll: ratation around the front-back axis (banking left/right)
    pub roll_deg: f64,
    /// Pitch: rotation around the left-right axis (node up/down)
    pub pitch_deg: f64,
    /// Yaw: rotation around the vertiacal axis (compass heading)
    pub yaw_deg: f64,
    pub roll_rate_deg_s: f64,
    pub pitch_rate_deg_s: f64,
    pub yaw_rate_deg_s: f64,
}

impl From<ATTITUDE_DATA> for Attitude {
    fn from(d: ATTITUDE_DATA) -> Self {
        const RAD_TO_DEG: f64 = 180.0 / std::f64::consts::PI;
        Self {
            time_boot: Duration::from_millis(d.time_boot_ms as u64),
            roll_deg: d.roll as f64 * RAD_TO_DEG,
            pitch_deg: d.pitch as f64 * RAD_TO_DEG,
            yaw_deg: d.yaw as f64 * RAD_TO_DEG,
            roll_rate_deg_s: d.rollspeed as f64 * RAD_TO_DEG,
            pitch_rate_deg_s: d.pitchspeed as f64 * RAD_TO_DEG,
            yaw_rate_deg_s: d.yawspeed as f64 * RAD_TO_DEG,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn attitude_converts_radians_to_degrees() {
        let raw = ATTITUDE_DATA {
            time_boot_ms: 1000,
            roll: 0.5,
            pitch: 0.3,
            yaw: 1.2,
            rollspeed: 0.1,
            pitchspeed: 0.2,
            yawspeed: 0.3,
            ..ATTITUDE_DATA::DEFAULT
        };
        let att: Attitude = raw.into();
        assert!((att.roll_deg - 28.6478898).abs() < 1e-3);
        assert!((att.pitch_deg - 17.1887339).abs() < 1e-3);
        assert!((att.yaw_deg - 68.7549354).abs() < 1e-3);
        assert!((att.roll_rate_deg_s - 5.729578).abs() < 1e-3);
        assert!((att.pitch_rate_deg_s - 11.459156).abs() < 1e-3);
        assert!((att.yaw_rate_deg_s - 17.188733).abs() < 1e-3);
    }
}
