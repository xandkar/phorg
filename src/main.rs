use std::path::PathBuf;

use clap::Parser;

use phorg::opt::Opt;

#[derive(Debug, Parser)]
struct Cli {
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

fn main() {
    let cli = Cli::parse();
    dbg!(&cli);
    phorg::organize(&cli.path, &cli.to_opt());
}
