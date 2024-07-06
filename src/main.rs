use std::path::PathBuf;

use clap::Parser;

#[derive(Debug, Parser)]
struct Cli {
    /// Specify log level, if any.
    #[clap(short, long = "log")]
    log_level: Option<tracing::Level>,

    /// Hash.
    #[clap(long, value_enum, default_value_t = phorg::hash::Hash::default())]
    hash: phorg::hash::Hash,

    /// Overwrite existing files.
    #[clap(short = 'f', long = "force", default_value_t = false)]
    force: bool,

    /// Type of files to filter for.
    #[clap(short, long = "type", name = "TYPE", value_enum, default_value_t = phorg::files::Typ::Image)]
    typ: phorg::files::Typ,

    /// Where to look for photo/video files.
    src_root: PathBuf,

    /// Where to create directory structure with found photo/video files.
    dst_root: PathBuf,

    /// What to do with the found photo files.
    #[clap(subcommand)]
    op: phorg::files::Op,
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    phorg::tracing_init(cli.log_level)?;
    phorg::files::organize(
        &cli.src_root,
        &cli.dst_root,
        &cli.op,
        cli.typ,
        cli.force,
        cli.hash,
    )?;
    Ok(())
}
