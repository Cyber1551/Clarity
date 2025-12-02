use tracing_subscriber::{fmt, EnvFilter};

/// Initializes the tracing subscriber for structured logging.
///
/// The log level can be controlled via the `RUST_LOG` environment variable.
/// Defaults to `info` level if not set.
pub fn init_logging() {
    let filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new("info"));

    fmt()
        .with_env_filter(filter)
        .with_target(true)
        .with_thread_ids(true)
        .with_file(true)
        .with_line_number(true)
        .init();
}
