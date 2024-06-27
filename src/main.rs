use std::path::PathBuf;

use clap::Parser;

#[derive(Debug, Parser)]
struct Cli {
    file: PathBuf,
}

fn main() {
    let cli = Cli::parse();
    dbg!(&cli);
}
