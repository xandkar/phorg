pub mod files;
pub mod hash;

mod exiftool;

pub fn tracing_init(level: Option<tracing::Level>) -> anyhow::Result<()> {
    use tracing_subscriber::{fmt, layer::SubscriberExt, EnvFilter, Layer};

    if let Some(level) = level {
        let layer_stderr = fmt::Layer::new()
            .with_writer(std::io::stderr)
            .with_ansi(true)
            .with_file(false)
            .with_line_number(true)
            .with_thread_ids(true)
            .with_filter(
                EnvFilter::from_default_env().add_directive(level.into()),
            );
        tracing::subscriber::set_global_default(
            tracing_subscriber::registry().with(layer_stderr),
        )?;
    }
    Ok(())
}

pub fn tracing_init_tests(level: tracing::Level) {
    let _ = tracing_subscriber::fmt()
        .with_test_writer()
        .with_ansi(true)
        .with_file(false)
        .with_line_number(true)
        .with_thread_ids(true)
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive(level.into()),
        )
        .try_init();
}
