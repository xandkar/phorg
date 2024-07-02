pub mod opt;

mod files;

use std::path::Path;

use anyhow::anyhow;
use exif::DateTime;

use opt::Opt;

pub fn organize(path: &Path, opt: &Opt) {
    explore(path, opt);
}

#[tracing::instrument(skip(opt))]
fn explore(path: &Path, opt: &Opt) {
    tracing::debug!(?path, ?opt, "Starting");
    let mut seen = 0;

    for path in files::find(path) {
        seen += 1;
        if let Err(error) = examine(&path, opt) {
            tracing::error!(?error, ?path, "Failed to examine file",);
        }
    }
    tracing::debug!(files_seen = seen, "Finished");
}

#[tracing::instrument(skip(opt))]
fn examine(path: &Path, opt: &Opt) -> anyhow::Result<()> {
    let file = std::fs::File::open(path)?;
    let mut bufreader = std::io::BufReader::new(&file);
    let exifreader = exif::Reader::new();
    let (class, value) = match exifreader.read_from_container(&mut bufreader) {
        Err(exif::Error::InvalidFormat(_)) => {
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
    if class != "Y" && !opt.show_failed {
        return Ok(());
    }
    if class == "Y" && opt.hide_success {
        return Ok(());
    }
    let row = [
        path.to_string_lossy().to_string().as_str(),
        class,
        value.as_str(),
    ]
    .join(&opt.sep);
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
