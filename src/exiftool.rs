use std::path::Path;

use crate::files::Timestamp;

#[derive(serde::Deserialize, Debug)]
struct Fields {
    #[serde(rename = "SourceFile")]
    _source_file: String,

    #[serde(
        rename = "CreateDate",
        deserialize_with = "exiftool_parse_date",
        default
    )]
    create_date: Option<Timestamp>,

    #[serde(
        rename = "CreationDate",
        deserialize_with = "exiftool_parse_date",
        default
    )]
    creation_date: Option<Timestamp>,

    #[serde(
        rename = "DateCreated",
        deserialize_with = "exiftool_parse_date",
        default
    )]
    date_created: Option<Timestamp>,

    #[serde(
        rename = "Datecreate",
        deserialize_with = "exiftool_parse_date",
        default
    )]
    date_create: Option<Timestamp>,

    #[serde(
        rename = "DateTimeCreated",
        deserialize_with = "exiftool_parse_date",
        default
    )]
    date_time_created: Option<Timestamp>,

    #[serde(
        rename = "DateTimeOriginal",
        deserialize_with = "exiftool_parse_date",
        default
    )]
    date_time_original: Option<Timestamp>,

    #[serde(
        rename = "TrackCreateDate",
        deserialize_with = "exiftool_parse_date",
        default
    )]
    track_create_date: Option<Timestamp>,

    #[serde(
        rename = "FileModifyDate",
        deserialize_with = "exiftool_parse_date",
        default
    )]
    file_modify_date: Option<Timestamp>,
}

#[tracing::instrument(skip_all)]
fn exiftool_parse_date<'de, D>(
    deserializer: D,
) -> Result<Option<Timestamp>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    use serde::Deserialize;

    let fmt = "%Y:%m:%d %H:%M:%S";
    match Option::deserialize(deserializer)? {
        None => Ok(None),
        Some(str) => chrono::NaiveDateTime::parse_from_str(str, fmt)
            .map(Some)
            .map_err(serde::de::Error::custom),
    }
}

#[tracing::instrument(skip_all)]
pub fn read_timestamp(path: &Path) -> Option<Timestamp> {
    let path = path.as_os_str().to_string_lossy().to_string();
    let out =
        cmd("exiftool", &["-json", "-CreateDate", "-DateCreated", &path])?;
    tracing::debug!(out = ?String::from_utf8_lossy(&out[..]), "Output raw");
    let parse_result = serde_json::from_slice::<Vec<Fields>>(&out[..]);
    tracing::debug!(?parse_result, "Output parsed");
    let mut fields_vec = parse_result.ok()?;
    if fields_vec.len() > 1 {
        tracing::warn!(
            ?fields_vec,
            "exiftool outputted more than 1 fields object"
        );
    }
    let Fields {
        _source_file,
        create_date,
        creation_date,
        date_created,
        date_create,
        date_time_created,
        date_time_original,
        track_create_date,
        file_modify_date,
    } = fields_vec.pop()?;
    date_time_original
        .or(date_time_created)
        .or(create_date)
        .or(date_created)
        .or(date_create)
        .or(creation_date)
        .or(track_create_date)
        .or(file_modify_date)
}

fn cmd(exe: &str, args: &[&str]) -> Option<Vec<u8>> {
    let out = std::process::Command::new(exe).args(args).output().ok()?;
    if out.status.success() {
        Some(out.stdout)
    } else {
        tracing::error!(
            ?exe,
            ?args,
            ?out,
            stderr = ?String::from_utf8_lossy(&out.stderr[..]),
            "Failed to execute command."
        );
        None
    }
}
