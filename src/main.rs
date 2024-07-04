use std::path::PathBuf;

use clap::Parser;

#[derive(Debug, Parser)]
struct Cli {
    /// Specify log level, if any.
    #[clap(short, long = "log")]
    log_level: Option<tracing::Level>,

    /// Overwrite existing files.
    #[clap(short = 'f', long = "force", default_value_t = false)]
    force: bool,

    /// Type of files to filter for.
    #[clap(short, long = "type", name = "TYPE", value_enum, default_value_t = phorg::Typ::Image)]
    typ: phorg::Typ,

    /// Where to look for photo/video files.
    src: PathBuf,

    /// Where to create directory structure with found photo/video files.
    dst: PathBuf,

    /// What to do with the found photo files.
    #[clap(subcommand)]
    op: phorg::Op,
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    phorg::tracing_init(cli.log_level)?;
    phorg::photos::organize(&cli.src, &cli.dst, &cli.op, cli.typ, cli.force)?;
    Ok(())
}
