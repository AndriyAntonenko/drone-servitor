use mavlink::dialects::ardupilotmega::{HEARTBEAT_DATA, MavModeFlag, MavState};
use std::time::{Duration, Instant};
use tokio::sync::{broadcast, watch};
use tracing::{info, warn};

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum FlightMode {
    Stabilize,
    AltHold,
    Loiter,
    Guided,
    Auto,
    Rtl,
    Land,
    Other(u32),
}

impl From<u32> for FlightMode {
    fn from(custom_mode: u32) -> Self {
        match custom_mode {
            0 => FlightMode::Stabilize,
            2 => FlightMode::AltHold,
            5 => FlightMode::Loiter,
            4 => FlightMode::Guided,
            3 => FlightMode::Auto,
            6 => FlightMode::Rtl,
            9 => FlightMode::Land,
            other => FlightMode::Other(other),
        }
    }
}

#[derive(Clone, Debug, Copy)]
pub struct Heartbeat {
    pub mode: FlightMode,
    pub armed: bool,
    pub system_healthy: bool,
}

impl From<HEARTBEAT_DATA> for Heartbeat {
    fn from(hd: HEARTBEAT_DATA) -> Self {
        Self {
            mode: FlightMode::from(hd.custom_mode),
            armed: hd
                .base_mode
                .contains(MavModeFlag::MAV_MODE_FLAG_SAFETY_ARMED),
            system_healthy: matches!(
                hd.system_status,
                MavState::MAV_STATE_ACTIVE | MavState::MAV_STATE_STANDBY
            ),
        }
    }
}

pub struct HeartbeatBus {
    tx: broadcast::Sender<Heartbeat>,
}

impl Default for HeartbeatBus {
    fn default() -> Self {
        let (tx, _rx) = broadcast::channel(16);
        Self { tx }
    }
}

impl HeartbeatBus {
    pub fn subscribe(&self) -> broadcast::Receiver<Heartbeat> {
        self.tx.subscribe()
    }

    pub fn send(&self, msg: Heartbeat) {
        // ignoring error on puprpose, because error retuned only of there is no listeners
        let _ = self.tx.send(msg);
    }
}

#[derive(Debug)]
pub struct HeartbeatState {
    pub mode: FlightMode,
    pub armed: bool,
    pub last_seen: Instant,
}

impl HeartbeatState {
    pub fn is_changed(&self, new_state: &Self) -> bool {
        self.armed != new_state.armed || self.mode != new_state.mode
    }
}

pub struct HeartbeatStateTracker {
    bus_rx: broadcast::Receiver<Heartbeat>,
    timeout: Duration,

    state_tx: watch::Sender<Option<HeartbeatState>>,
}

impl HeartbeatStateTracker {
    pub fn new(
        timeout: Duration,
        heartbeat_bus_rx: broadcast::Receiver<Heartbeat>,
    ) -> (Self, watch::Receiver<Option<HeartbeatState>>) {
        let (state_tx, state_rx) = watch::channel(None);
        (
            Self {
                timeout,
                bus_rx: heartbeat_bus_rx,
                state_tx,
            },
            state_rx,
        )
    }

    pub async fn start(mut self) {
        tokio::spawn(async move {
            let sleep = tokio::time::sleep(self.timeout);
            tokio::pin!(sleep);

            loop {
                tokio::select! {
                    msg_or_err = self.bus_rx.recv() => {
                        match msg_or_err {
                            Ok(hb) => {
                                let new = HeartbeatState { mode: hb.mode, armed: hb.armed, last_seen: Instant::now() };
                                let prev = self.state_tx.borrow();
                                if let Some(prev) = prev.as_ref() && prev.is_changed(&new){
                                    info!(mode = ?new.mode, armed = ?new.armed, "heartbeat state has changed");
                                }

                                drop(prev);
                                if self.state_tx.send(Some(new)).is_err() {
                                    warn!("failed to send new heartbeat state, breaking");
                                    break;
                                }

                                sleep.as_mut().reset(tokio::time::Instant::now() + self.timeout);
                            },
                            Err(broadcast::error::RecvError::Lagged(n)) => {
                                warn!(missed = n, "heartbeat receiver lagged - {n} messages dropped, state transitions may have been missed");
                            },
                            Err(broadcast::error::RecvError::Closed) => break,
                        }

                    }
                    () = &mut sleep => {
                        let prev = self.state_tx.borrow();
                        if let Some(prev) = prev.as_ref() {
                            warn!(
                                timeout_secs = self.timeout.as_secs(),
                                last_seen_ago_secs =  Instant::now().duration_since(prev.last_seen).as_secs_f64(),
                                "no heartbeat received within timeout - check MAVLink connection / SITL process"
                            );
                        }
                        sleep.as_mut().reset(tokio::time::Instant::now() + self.timeout);
                    }
                }
            }
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn flight_mode_maps_known_custom_modes() {
        assert_eq!(FlightMode::from(0), FlightMode::Stabilize);
        assert_eq!(FlightMode::from(4), FlightMode::Guided);
        assert_eq!(FlightMode::from(6), FlightMode::Rtl);
        assert_eq!(FlightMode::from(42), FlightMode::Other(42));
    }

    #[test]
    fn heartbeat_detects_armed_and_healthy() {
        let raw = HEARTBEAT_DATA {
            custom_mode: 4,
            base_mode: MavModeFlag::MAV_MODE_FLAG_SAFETY_ARMED,
            system_status: MavState::MAV_STATE_ACTIVE,
            ..HEARTBEAT_DATA::DEFAULT
        };
        let hb: Heartbeat = raw.into();
        assert_eq!(hb.mode, FlightMode::Guided);
        assert!(hb.armed);
        assert!(hb.system_healthy);
    }

    #[test]
    fn heartbeat_detects_disarmed_and_unhealthy() {
        let raw = HEARTBEAT_DATA {
            custom_mode: 9,
            base_mode: MavModeFlag::MAV_MODE_FLAG_MANUAL_INPUT_ENABLED,
            system_status: MavState::MAV_STATE_CRITICAL,
            ..HEARTBEAT_DATA::DEFAULT
        };
        let hb: Heartbeat = raw.into();
        assert_eq!(hb.mode, FlightMode::Land);
        assert!(!hb.armed);
        assert!(!hb.system_healthy);
    }

    #[tokio::test]
    async fn heartbeat_bus_subscribe_and_send() {
        let bus = HeartbeatBus::default();

        let mut rx = bus.subscribe();
        let raw = HEARTBEAT_DATA {
            custom_mode: 9,
            base_mode: MavModeFlag::MAV_MODE_FLAG_MANUAL_INPUT_ENABLED,
            system_status: MavState::MAV_STATE_CRITICAL,
            ..HEARTBEAT_DATA::DEFAULT
        };

        let msg: Heartbeat = raw.into();

        bus.send(msg);

        let received = rx.recv().await.unwrap();
        assert_eq!(msg.system_healthy, received.system_healthy);
        assert_eq!(msg.mode, received.mode);
        assert_eq!(msg.armed, received.armed);
    }
}
