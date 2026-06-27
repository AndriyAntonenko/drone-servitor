use tracing_subscriber::{fmt, layer::SubscriberExt, util::SubscriberInitExt};

pub fn setup_tracing() -> tracing_appender::non_blocking::WorkerGuard {
    let file_appender = tracing_appender::rolling::never("logs", "servitor.log");
    let (non_blocking, guard) = tracing_appender::non_blocking(file_appender);
    tracing_subscriber::registry()
        .with(fmt::layer())
        .with(fmt::layer().with_writer(non_blocking))
        .init();

    guard
}
