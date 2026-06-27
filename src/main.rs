use drone_servitor::telemetry::{
    Telemetry,
    heartbeat::{HeartbeatBus, HeartbeatStateTracker},
    parse_known_telemetry,
};
use mavlink::dialects::ardupilotmega::MavMessage;
use tracing::{info, trace, warn};

#[derive(serde::Deserialize)]
struct Config {
    pub connection_string: String,
}

async fn load_config(path: &str) -> anyhow::Result<Config> {
    let config_str = tokio::fs::read_to_string(path).await?;
    let conf = toml::from_str::<Config>(config_str.as_str())?;
    anyhow::Ok(conf)
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();

    let config = load_config("config.toml").await?;

    let conn = mavlink::connect_async::<MavMessage>(config.connection_string.as_str()).await?;

    info!("listening on {}", config.connection_string.as_str());

    let heartbeat_bus = HeartbeatBus::default();
    let (heartbeat_state_tracker, _heartbeat_state_rx) =
        HeartbeatStateTracker::new(std::time::Duration::from_secs(3), heartbeat_bus.subscribe());

    heartbeat_state_tracker.start().await;
    info!("heartbeat monitor started");
    loop {
        match conn.recv().await {
            Ok((_, msg)) => {
                if let Some(tm) = parse_known_telemetry(msg) {
                    trace!("received telemetry: {:?}", tm);
                    if let Telemetry::Heartbeat(hb) = tm {
                        heartbeat_bus.send(hb);
                    }
                }
            }
            Err(err) => {
                warn!(error = ?err, "recv error");
            }
        }
    }
}
