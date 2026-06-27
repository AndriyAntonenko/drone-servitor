pub mod attitude;
pub mod ekf;
pub mod heartbeat;
pub mod position;
pub mod sys_status;

use mavlink::dialects::ardupilotmega::MavMessage;

#[derive(Debug, Clone, Copy)]
pub enum Telemetry {
    Heartbeat(heartbeat::Heartbeat),
    Attitude(attitude::Attitude),
    Position(position::Position),
    SystemStatus(sys_status::SystemStatus),
    EkfHealth(ekf::EkfHealth),
}

pub fn parse_known_telemetry(mav_msg: MavMessage) -> Option<Telemetry> {
    match mav_msg {
        MavMessage::HEARTBEAT(data) => Some(Telemetry::Heartbeat(heartbeat::Heartbeat::from(data))),
        MavMessage::ATTITUDE(data) => Some(Telemetry::Attitude(attitude::Attitude::from(data))),
        MavMessage::GLOBAL_POSITION_INT(data) => {
            Some(Telemetry::Position(position::Position::from(data)))
        }
        MavMessage::SYS_STATUS(data) => Some(Telemetry::SystemStatus(
            sys_status::SystemStatus::from(data),
        )),
        MavMessage::EKF_STATUS_REPORT(data) => {
            Some(Telemetry::EkfHealth(ekf::EkfHealth::from(data)))
        }
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use mavlink::dialects::ardupilotmega::{HEARTBEAT_DATA, MavState, SYSTEM_TIME_DATA};

    #[test]
    fn parse_known_telemetry_dispatches_heartbeat() {
        let raw = HEARTBEAT_DATA {
            system_status: MavState::MAV_STATE_ACTIVE,
            ..HEARTBEAT_DATA::DEFAULT
        };
        let telemetry = parse_known_telemetry(MavMessage::HEARTBEAT(raw));
        assert!(matches!(telemetry, Some(Telemetry::Heartbeat(_))));
    }

    #[test]
    fn parse_known_telemetry_ignores_unmapped_messages() {
        let telemetry = parse_known_telemetry(MavMessage::SYSTEM_TIME(SYSTEM_TIME_DATA::DEFAULT));
        assert!(telemetry.is_none());
    }
}
