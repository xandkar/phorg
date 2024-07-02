use std::path::PathBuf;

use clap::Parser;

use phorg::opt::Opt;

#[derive(Debug, Parser)]
struct Cli {
    /// Specify log level, if any.
    #[clap(short, long = "log")]
    log_level: Option<tracing::Level>,

    /// Output table field separator.
    #[clap(short, long, default_value = "|")]
    sep: String,

    /// Show files with failed lookups.
    #[clap(short = 'f', long = "failed", default_value_t = false)]
    show_failed: bool,

    /// Hide files with successful lookups.
    #[clap(short = 'H', long = "hide", default_value_t = false)]
    hide_success: bool,

    #[clap(default_value = ".")]
    path: PathBuf,
}

impl Cli {
    fn to_opt(&self) -> Opt {
        Opt {
            sep: self.sep.to_string(),
            show_failed: self.show_failed,
            hide_success: self.hide_success,
        }
    }
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    tracing_init(cli.log_level)?;
    tracing::info!(?cli, "Starting");
    phorg::organize(&cli.path, &cli.to_opt());
    Ok(())
}

fn tracing_init(level: Option<tracing::Level>) -> anyhow::Result<()> {
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
