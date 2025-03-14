use tracing_subscriber::{self, fmt, layer::SubscriberExt as _, util::SubscriberInitExt as _, EnvFilter};

pub fn setup_tracing_subscriber() -> anyhow::Result<()>  {
    // For now we are simply working with the defaults.
    let fmt_layer = fmt::layer();
    let filter = EnvFilter::from_default_env();

    tracing_subscriber::registry()
        .with(filter)
        .with(fmt_layer)
        .init();

    Ok(())
}