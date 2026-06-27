use mavlink::dialects::ardupilotmega::{MavSysStatusSensor, SYS_STATUS_DATA};

#[derive(Debug, Clone, Copy)]
pub struct SystemStatus {
    pub cpu_load_pct: f64,
    pub battery_voltage_v: Option<f64>,
    pub battery_voltage_a: Option<f64>,
    pub battery_remaining_pct: Option<u8>,
    pub gps_healthy: bool,
}

impl From<SYS_STATUS_DATA> for SystemStatus {
    fn from(d: SYS_STATUS_DATA) -> Self {
        Self {
            cpu_load_pct: d.load as f64 / 10.0,
            battery_voltage_v: if d.voltage_battery == u16::MAX {
                None
            } else {
                Some(d.voltage_battery as f64 / 1000.0)
            },
            battery_voltage_a: if d.current_battery < 0 {
                None
            } else {
                Some(d.current_battery as f64 / 100.0)
            },
            battery_remaining_pct: if d.battery_remaining < 0 {
                None
            } else {
                Some(d.battery_remaining as u8)
            },
            gps_healthy: d
                .onboard_control_sensors_health
                .contains(MavSysStatusSensor::MAV_SYS_STATUS_SENSOR_GPS),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sys_status_converts_normal_values() {
        let raw = SYS_STATUS_DATA {
            load: 550,              // 55.0%
            voltage_battery: 12000, // 12.0 V
            current_battery: 250,   // 2.5 A
            battery_remaining: 80,
            onboard_control_sensors_health: MavSysStatusSensor::MAV_SYS_STATUS_SENSOR_GPS,
            ..SYS_STATUS_DATA::DEFAULT
        };
        let status: SystemStatus = raw.into();
        assert!((status.cpu_load_pct - 55.0).abs() < 1e-6);
        assert_eq!(status.battery_voltage_v, Some(12.0));
        assert_eq!(status.battery_voltage_a, Some(2.5));
        assert_eq!(status.battery_remaining_pct, Some(80));
        assert!(status.gps_healthy);
    }

    #[test]
    fn sys_status_treats_sentinels_as_unavailable() {
        let raw = SYS_STATUS_DATA {
            voltage_battery: u16::MAX,
            current_battery: -1,
            battery_remaining: -1,
            ..SYS_STATUS_DATA::DEFAULT
        };
        let status: SystemStatus = raw.into();
        assert_eq!(status.battery_voltage_v, None);
        assert_eq!(status.battery_voltage_a, None);
        assert_eq!(status.battery_remaining_pct, None);
        assert!(!status.gps_healthy);
    }
}
