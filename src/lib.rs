pub mod photos;

mod files;

// TODO Keep clap/CLI-specific stuff out of lib code.
#[derive(clap::Subcommand, Debug)]
pub enum Op {
    /// Dry run. Just print what would be done.
    Show {
        /// Output table field separator.
        #[clap(short, long, default_value = "|")]
        sep: String,
    },

    /// Copy into the directory structure in dst (i.e. preserve the original files in src).
    Copy,

    /// Move into the directory structure in dst (i.e. remove the original files from src).
    Move,
}

#[derive(Debug, Clone, Copy, clap::ValueEnum)]
pub enum Typ {
    Image,
    Video,
}

pub fn tracing_init(level: Option<tracing::Level>) -> anyhow::Result<()> {
    use tracing_subscriber::{fmt, layer::SubscriberExt, EnvFilter, Layer};

    if let Some(level) = level {
        let layer_stderr = fmt::Layer::new()
            .with_writer(std::io::stderr)
            .with_ansi(true)
            .with_file(false)
            .with_line_number(true)
            .with_thread_ids(true)
            .with_filter(EnvFilter::from_default_env().add_directive(level.into()));
        tracing::subscriber::set_global_default(tracing_subscriber::registry().with(layer_stderr))?;
    }
    Ok(())
}
