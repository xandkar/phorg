use std::path::PathBuf;

use clap::Parser;

#[derive(Debug, Parser)]
struct Cli {
    /// Specify log level, if any.
    #[clap(short, long = "log")]
    log_level: Option<tracing::Level>,

    /// Where to look for photo files.
    #[clap(long)]
    src: PathBuf,

    /// Where to create directory structure with found photo files.
    #[clap(long)]
    dst: PathBuf,

    /// What to do with the found photo files.
    #[clap(subcommand)]
    op: phorg::Op,
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    phorg::tracing_init(cli.log_level)?;
    phorg::photos::organize(&cli.src, &cli.dst, &cli.op)?;
    Ok(())
}
