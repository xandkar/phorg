use std::{
    collections::VecDeque,
    fs,
    path::{Path, PathBuf},
};

use anyhow::anyhow;
use clap::Parser;
use exif::DateTime;

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
    dir: PathBuf,
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    dbg!(&cli);

    let mut seen = 0;
    let mut dir_queue: VecDeque<PathBuf> = VecDeque::new();
    dir_queue.push_back(cli.dir);
    while let Some(dir) = dir_queue.pop_front() {
        for entry_result in fs::read_dir(dir)? {
            let entry = entry_result?;
            let path = entry.path();
            match entry.file_type()? {
                typ if typ.is_file() => {
                    seen += 1;
                    examine(&path, &cli.sep, cli.show_failed, cli.hide_success)?;
                }
                typ if typ.is_dir() => dir_queue.push_back(path),
                _ => (),
            }
        }
    }
    eprintln!("[debug] Seen {} files.", seen);
    Ok(())
}

fn examine(path: &Path, sep: &str, show_failed: bool, hide_success: bool) -> anyhow::Result<()> {
    let file = std::fs::File::open(path)?;
    let mut bufreader = std::io::BufReader::new(&file);
    let exifreader = exif::Reader::new();
    let (class, value) = match exifreader.read_from_container(&mut bufreader) {
        Err(exif::Error::InvalidFormat(_)) => {
            // eprintln!("[warn] Not a supported image file: {:?}", path);
            // Skip non-image files.
            return Ok(());
        }
        Err(exif::Error::BlankValue(_msg)) => ("N", "--".to_string()),
        Err(exif::Error::NotFound(_file_type)) => ("N", "--".to_string()),
        Err(error) => ("E", error.to_string()),
        Ok(exif) => match get_date_time_original(&exif) {
            Err(error) => ("E", error.to_string()),
            Ok(None) => ("N", "--".to_string()),
            Ok(Some(timestamp)) => ("Y", timestamp.to_string()),
        },
    };
    if class != "Y" && !show_failed {
        return Ok(());
    }
    if class == "Y" && hide_success {
        return Ok(());
    }
    let row = [
        path.to_string_lossy().to_string().as_str(),
        class,
        value.as_str(),
    ]
    .join(sep);
    println!("{}", row);
    Ok(())
}

// Ref: exif::tag::d_datetime (private).
fn get_date_time_original(exif: &exif::Exif) -> anyhow::Result<Option<exif::DateTime>> {
    match exif.get_field(exif::Tag::DateTimeOriginal, exif::In::PRIMARY) {
        None => Ok(None),
        Some(field) => match &field.value {
            exif::Value::Ascii(ref data) => match data.first() {
                None => Ok(None),
                Some(data) => {
                    let dt = DateTime::from_ascii(&data[..])?;
                    Ok(Some(dt))
                }
            },
            value => Err(anyhow!(
                "DateTimeOriginal field value is not ASCII: {:?}",
                value
            )),
        },
    }
}
