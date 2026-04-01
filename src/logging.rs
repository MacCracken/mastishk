//! Optional logging initialisation.

/// Initialise logging with default level `info`.
pub fn init() {
    init_with_level("info");
}

/// Initialise logging with a custom default level.
pub fn init_with_level(default_level: &str) {
    use tracing_subscriber::EnvFilter;
    use tracing_subscriber::fmt;
    use tracing_subscriber::prelude::*;

    let filter = EnvFilter::try_from_env("MASTISHK_LOG")
        .unwrap_or_else(|_| EnvFilter::new(default_level));

    let _ = tracing_subscriber::registry()
        .with(fmt::layer().with_target(true).with_thread_ids(true))
        .with(filter)
        .try_init();
}
