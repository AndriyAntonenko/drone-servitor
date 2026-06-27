use mavlink::dialects::ardupilotmega::{EKF_STATUS_REPORT_DATA, EkfStatusFlags};

#[derive(Debug, Clone, Copy)]
pub struct EkfHealth {
    pub attitude_ok: bool,
    pub velocity_ok: bool,
    pub position_horiz_ok: bool,
    pub position_vert_ok: bool,
    pub using_gps: bool,
}

impl From<EKF_STATUS_REPORT_DATA> for EkfHealth {
    fn from(d: EKF_STATUS_REPORT_DATA) -> Self {
        Self {
            attitude_ok: d.flags.contains(EkfStatusFlags::EKF_ATTITUDE),
            velocity_ok: d.flags.contains(EkfStatusFlags::EKF_VELOCITY_HORIZ),
            position_horiz_ok: d.flags.contains(EkfStatusFlags::EKF_POS_HORIZ_ABS),
            position_vert_ok: d.flags.contains(EkfStatusFlags::EKF_POS_VERT_ABS),
            using_gps: !d.flags.contains(EkfStatusFlags::EKF_CONST_POS_MODE),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ekf_reports_all_estimates_good_when_flags_set() {
        let raw = EKF_STATUS_REPORT_DATA {
            flags: EkfStatusFlags::EKF_ATTITUDE
                | EkfStatusFlags::EKF_VELOCITY_HORIZ
                | EkfStatusFlags::EKF_POS_HORIZ_ABS
                | EkfStatusFlags::EKF_POS_VERT_ABS,
            ..EKF_STATUS_REPORT_DATA::DEFAULT
        };
        let health: EkfHealth = raw.into();
        assert!(health.attitude_ok);
        assert!(health.velocity_ok);
        assert!(health.position_horiz_ok);
        assert!(health.position_vert_ok);
        assert!(health.using_gps);
    }

    #[test]
    fn ekf_const_pos_mode_means_not_using_gps() {
        let raw = EKF_STATUS_REPORT_DATA {
            flags: EkfStatusFlags::EKF_ATTITUDE | EkfStatusFlags::EKF_CONST_POS_MODE,
            ..EKF_STATUS_REPORT_DATA::DEFAULT
        };
        let health: EkfHealth = raw.into();
        assert!(health.attitude_ok);
        assert!(!health.velocity_ok);
        assert!(!health.position_horiz_ok);
        assert!(!health.position_vert_ok);
        assert!(!health.using_gps);
    }
}
