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
    _file_modify_date: Option<Timestamp>,
}

#[tracing::instrument(skip_all)]
fn exiftool_parse_date<'de, D>(
    deserializer: D,
) -> Result<Option<Timestamp>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    use serde::Deserialize;

    match Option::deserialize(deserializer)? {
        None => Ok(None),
        Some(data) => naive_date_time_parse(data)
            .map(Some)
            .map_err(serde::de::Error::custom),
    }
}

fn naive_date_time_parse(
    data: &str,
) -> chrono::format::ParseResult<Timestamp> {
    const FMT: &str = "%Y:%m:%d %H:%M:%S";
    let mut data = data.to_owned();
    // Drop timezone if it is present:
    if let Some(pos) = data.find(['+', '-']) {
        data.truncate(pos);
    }
    // Drop subseconds if they're present:
    if let Some(pos) = data.find(['.']) {
        data.truncate(pos);
    }
    chrono::NaiveDateTime::parse_from_str(&data, FMT)
}

#[tracing::instrument(skip_all)]
pub fn read_timestamp(path: &Path) -> Option<Timestamp> {
    let path = path.as_os_str().to_string_lossy().to_string();
    let out = cmd("exiftool", &["-json", &path])?;
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
        _file_modify_date,
    } = fields_vec.pop()?;
    date_time_original
        .or(date_time_created)
        .or(creation_date)
        .or(create_date)
        .or(date_created)
        .or(date_create)
        .or(track_create_date)
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn t_naive_date_time_parse() {
        assert!(naive_date_time_parse("").is_err());
        assert!(naive_date_time_parse("-200:88:90 1:2:3").is_err());
        assert_eq!(
            chrono::NaiveDateTime::new(
                chrono::NaiveDate::from_ymd_opt(2022, 10, 3).unwrap(),
                chrono::NaiveTime::from_hms_opt(17, 52, 16).unwrap(),
            ),
            naive_date_time_parse("2022:10:03 17:52:16.597752928733826Z")
                .unwrap()
        );
        assert_eq!(
            chrono::NaiveDateTime::new(
                chrono::NaiveDate::from_ymd_opt(2022, 10, 3).unwrap(),
                chrono::NaiveTime::from_hms_opt(17, 52, 16).unwrap(),
            ),
            naive_date_time_parse("2022:10:03 17:52:16").unwrap()
        );
        assert_eq!(
            chrono::NaiveDateTime::new(
                chrono::NaiveDate::from_ymd_opt(2022, 10, 3).unwrap(),
                chrono::NaiveTime::from_hms_opt(17, 52, 16).unwrap(),
            ),
            naive_date_time_parse("2022:10:03 17:52:16+4:00").unwrap()
        );
        assert_eq!(
            chrono::NaiveDateTime::new(
                chrono::NaiveDate::from_ymd_opt(2022, 10, 3).unwrap(),
                chrono::NaiveTime::from_hms_opt(17, 52, 16).unwrap(),
            ),
            naive_date_time_parse("2022:10:03 17:52:16-7:00").unwrap()
        );
    }
}
