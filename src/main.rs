use std::path::PathBuf;

use clap::Parser;

#[derive(Debug, Parser)]
#[command(version, about)]
struct Cli {
    /// Specify log level, if any.
    #[clap(short, long = "log", default_value_t = tracing::Level::WARN)]
    log_level: tracing::Level,

    /// Hash.
    #[clap(long, value_enum, default_value_t = phorg::hash::Hash::default())]
    hash: phorg::hash::Hash,

    /// Overwrite existing files.
    #[clap(short = 'f', long = "force", default_value_t = false)]
    force: bool,

    /// Don't try falling back on exiftool if we fail to extract Exif data directly.
    #[clap(long, default_value_t = false)]
    no_exiftool: bool,

    /// Show progress bar (when copying or moving, but never when showing).
    /// NOTE: May conflict with logging output, so may need to set the log
    /// level to error to avoid screen noise.
    #[clap(short = 'p', long = "progress", default_value_t = false)]
    show_progress: bool,

    /// Process only this file type, otherwise all supported will be processed.
    #[clap(short, long = "type", name = "TYPE", value_enum)]
    typ: Option<phorg::files::Typ>,

    /// Image subdirectory under DST_ROOT.
    #[clap(long, default_value = "img")]
    img_dir: String,

    /// Video subdirectory under DST_ROOT.
    #[clap(long, default_value = "vid")]
    vid_dir: String,

    /// Where to look for photo/video files.
    src_root: PathBuf,

    /// Where to create directory structure with found photo/video files.
    dst_root: PathBuf,

    /// What to do with the found photo files.
    #[clap(subcommand)]
    op: phorg::files::Op,
}

fn main() -> anyhow::Result<()> {
    human_panic_setup();
    let cli = Cli::parse();
    phorg::tracing_init(Some(cli.log_level))?;
    let use_exiftool = !cli.no_exiftool;
    phorg::files::organize(
        &cli.src_root,
        &cli.dst_root,
        &cli.op,
        &cli.img_dir,
        &cli.vid_dir,
        cli.typ,
        cli.force,
        use_exiftool,
        cli.show_progress,
        cli.hash,
    )?;
    Ok(())
}

fn human_panic_setup() {
    macro_rules! repo {
        () => {
            env!("CARGO_PKG_REPOSITORY")
        };
    }
    human_panic::setup_panic!(human_panic::Metadata::new(
        env!("CARGO_PKG_NAME"),
        env!("CARGO_PKG_VERSION")
    )
    .authors(env!("CARGO_PKG_AUTHORS"))
    .homepage(repo!())
    .support(concat!("- Submit an issue at ", repo!(), "/issues")));
}
