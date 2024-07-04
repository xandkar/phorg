use std::{
    fs::{self, File},
    io,
    path::{Path, PathBuf},
};

use anyhow::Context;

use crate::{files, Op, Typ};

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("DateTimeOriginal field value is not ASCII: {0:?}")]
    DateTimeOriginalValueNotAscii(exif::Value),

    #[error("Unsupported file type: {0:?}")]
    InvalidFormat(PathBuf),

    #[error("IO error: {0:?}")]
    Io(#[from] io::Error),

    #[error("Exif error: {0:?}")]
    Other(#[from] exif::Error),
}

pub type Timestamp = chrono::NaiveDateTime;

#[derive(Debug)]
pub struct Photo {
    pub src: PathBuf,
    pub dst: Option<PathBuf>,
    pub timestamp: Option<Timestamp>,
}

impl Photo {
    pub fn read(path: &Path, typ: Typ) -> Result<Self, Error> {
        match typ {
            Typ::Image => Self::read_image(path),
            Typ::Video => Self::read_video(path),
        }
    }

    pub fn read_video(path: &Path) -> Result<Self, Error> {
        let path = path.to_path_buf();
        let file = std::fs::File::open(&path)?;
        let timestamp: Option<Timestamp> = nom_exif::parse_metadata(file).ok().and_then(|pairs| {
            pairs
                .iter()
                .find(|(k, _)| k == "com.apple.quicktime.creationdate")
                .map(|(_, v)| v)
                .and_then(|entry| match entry {
                    nom_exif::EntryValue::Time(t) => Some(t),
                    _ => None,
                })
                .map(|t| t.naive_local())
        });
        let dst = timestamp.and_then(|t| dst(&path, t));
        let selph = Self {
            src: path,
            dst,
            timestamp,
        };
        Ok(selph)
    }

    pub fn read_image(path: &Path) -> Result<Self, Error> {
        let path = path.to_path_buf();
        let file = std::fs::File::open(&path)?;
        let mut bufreader = std::io::BufReader::new(&file);
        let exifreader = exif::Reader::new();
        match exifreader.read_from_container(&mut bufreader) {
            Err(exif::Error::InvalidFormat(_)) => Err(Error::InvalidFormat(path)),
            Err(exif::Error::BlankValue(msg)) => {
                tracing::error!(?path, ?msg, "Blank value");
                Ok(Self {
                    src: path,
                    dst: None,
                    timestamp: None,
                })
            }
            Err(exif::Error::NotFound(file_type)) => {
                tracing::error!(?path, ?file_type, "EXIF data not found");
                Ok(Self {
                    src: path,
                    dst: None,
                    timestamp: None,
                })
            }
            Err(error) => {
                tracing::error!(?path, ?error, "Read failure");
                Err(Error::Other(error))
            }
            Ok(exif) => {
                let timestamp = get_date_time_original(&exif)?
                    .as_ref()
                    .and_then(date_time_exif_to_chrono);
                if timestamp.is_none() {
                    tracing::error!(?path, "Timestamp data not found");
                }
                let dst = timestamp.and_then(|ts| dst(&path, ts));
                Ok(Self {
                    src: path,
                    dst,
                    timestamp,
                })
            }
        }
    }

    fn show(&self, sep: &str) {
        let src = self.src.to_string_lossy().to_string();
        let timestamp = self
            .timestamp
            .as_ref()
            .map_or("--".to_string(), |ts| ts.to_string());
        let dst = self
            .dst
            .as_ref()
            .map_or("--".to_string(), |dst| dst.to_string_lossy().to_string());
        let row = [src, timestamp, dst].join(sep);
        println!("{}", row);
    }

    #[tracing::instrument]
    fn organize(&self, dst_dir: &Path, permanently: bool) -> anyhow::Result<()> {
        tracing::info!("Organizing");
        let src = self.src.as_path();
        let dst = self.dst.as_ref().map(|dst_file| dst_dir.join(dst_file));
        if let Some(dst_parent) = dst.as_ref().and_then(|path| path.parent()) {
            fs::create_dir_all(dst_parent)
                .context(format!("Failed to create parent dir: {:?}", dst_parent))?;
        }
        match dst {
            None => {
                tracing::warn!("Ignoring. dst undetermined.");
            }
            Some(dst) if permanently => {
                tracing::info!("Moving");
                fs::rename(src, &dst).context(format!(
                    "Failed to rename file. src={:?}. dst={:?}",
                    src, &dst
                ))?;
            }
            Some(dst) => {
                tracing::info!("Copying");
                fs::copy(src, &dst).context(format!(
                    "Failed to copy file. src={:?}. dst={:?}",
                    src, &dst
                ))?;
            }
        }
        tracing::info!("Done");
        Ok(())
    }
}

#[tracing::instrument]
pub fn find(path: &Path, typ: Typ) -> impl Iterator<Item = Photo> {
    files::find(path)
        .filter(move |path| {
            let is_type = match typ {
                Typ::Image => file_is_image(path),
                Typ::Video => file_is_video(path),
            };
            tracing::debug!(?path, ?typ, ?is_type, "Type filter");
            is_type
        })
        .filter_map(move |path| {
            let result_nom: anyhow::Result<Vec<(String, nom_exif::EntryValue)>> = File::open(&path)
                .map_err(anyhow::Error::from)
                .and_then(|f| {
                    let data = nom_exif::parse_metadata(f)?;
                    Ok(data)
                });
            let result = Photo::read(path.as_path(), typ);
            tracing::debug!(?result, ?result_nom, "Fetched");
            result.ok()
        })
}

fn file_is_image(path: &Path) -> bool {
    fetch_type(path).is_some_and(type_is_image)
}

fn file_is_video(path: &Path) -> bool {
    fetch_type(path).is_some_and(type_is_video)
}

fn type_is_image(ty: infer::Type) -> bool {
    matches!(ty.matcher_type(), infer::MatcherType::Image)
}

fn type_is_video(ty: infer::Type) -> bool {
    matches!(ty.matcher_type(), infer::MatcherType::Video)
}

fn fetch_type(path: &Path) -> Option<infer::Type> {
    infer::get_from_path(path)
        .map_err(|error| {
            tracing::error!(?path, ?error, "Failed to read file.");
        })
        .ok()
        .flatten()
}

#[tracing::instrument(skip_all)]
pub fn organize(src: &Path, dst: &Path, op: &Op, typ: Typ) -> anyhow::Result<()> {
    tracing::info!(?op, ?src, ?dst, "Starting");
    let src = src
        .canonicalize()
        .context(format!("Failed to canonicalize src path: {:?}", src))?;
    if !dst
        .try_exists()
        .context(format!("Failed to check existence of dst path: {:?}", &dst))?
    {
        tracing::info!(path = ?dst, "Dst dir does not exist. Creating.");
        fs::create_dir_all(dst).context(format!("Failed to create dst dir: {:?}", dst))?;
    }
    let dst = dst
        .canonicalize()
        .context(format!("Failed to canonicalize dst path: {:?}", dst))?;
    tracing::info!(?src, ?dst, "Canonicalized");
    for photo in find(&src, typ) {
        match op {
            Op::Show { sep } => photo.show(sep),
            Op::Copy => photo.organize(&dst, false)?,
            Op::Move => photo.organize(&dst, true)?,
        }
    }
    tracing::info!("Finished");
    Ok(())
}

// Ref: exif::tag::d_datetime (private).
fn get_date_time_original(exif: &exif::Exif) -> Result<Option<exif::DateTime>, Error> {
    match exif.get_field(exif::Tag::DateTimeOriginal, exif::In::PRIMARY) {
        None => Ok(None),
        Some(field) => match &field.value {
            exif::Value::Ascii(ref data) => match data.first() {
                None => Ok(None),
                Some(data) => {
                    let dt_opt = exif::DateTime::from_ascii(&data[..])
                        .map_err(|error| {
                            tracing::error!(?error, "Failed to read DateTimeOriginal field");
                        })
                        .ok();
                    Ok(dt_opt)
                }
            },
            value => Err(Error::DateTimeOriginalValueNotAscii(value.clone())),
        },
    }
}

fn date_time_exif_to_chrono(dt: &exif::DateTime) -> Option<chrono::NaiveDateTime> {
    let time = chrono::NaiveTime::from_hms_opt(
        u32::from(dt.hour),
        u32::from(dt.minute),
        u32::from(dt.second),
    )?;
    let date = chrono::NaiveDate::from_ymd_opt(
        i32::from(dt.year),
        u32::from(dt.month),
        u32::from(dt.day),
    )?;
    Some(chrono::NaiveDateTime::new(date, time))
}

fn dst(src: &Path, ts: Timestamp) -> Option<PathBuf> {
    match (src.file_stem(), src.extension()) {
        (Some(stem_old), Some(extension)) => {
            use chrono::{Datelike, Timelike}; // Access timestamp fields.

            let year = format!("{:02}", ts.year());
            let month = format!("{:02}", ts.month());
            let day = format!("{:02}", ts.day());
            let hour = format!("{:02}", ts.hour());
            let minute = format!("{:02}", ts.minute());
            let second = format!("{:02}", ts.second());
            let stem_old = stem_old
                .to_str()
                .map_or(String::new(), |x| format!("--{}", x));
            let stem_new = format!(
                "{}-{}-{}--{}:{}:{}{}",
                &year, &month, &day, &hour, &minute, &second, &stem_old
            );
            let name_new = PathBuf::from(stem_new).with_extension(extension);
            let dir: PathBuf = [&year, &month, &day].iter().collect();
            Some(dir.join(name_new))
        }
        (None, None) => {
            tracing::error!(path = ?src, "Missing stem and extension");
            None
        }
        (None, Some(_)) => {
            tracing::error!(path = ?src, "Missing stem");
            None
        }
        (Some(_), None) => {
            tracing::error!(path = ?src, "Missing extension");
            None
        }
    }
}
