use std::{
    collections::VecDeque,
    ffi::OsStr,
    fs,
    path::{Path, PathBuf},
};

use anyhow::Context;
use rayon::prelude::*;

use crate::{exiftool, hash::Hash};

// TODO Keep clap/CLI-specific stuff out of lib code.
#[derive(clap::Subcommand, Debug)]
pub enum Op {
    /// Dry run. Just print what would be done.
    Show,

    /// Copy into the directory structure in dst (i.e. preserve the original files in src).
    Copy,

    /// Move into the directory structure in dst (i.e. remove the original files from src).
    Move,
}

#[derive(Debug, Clone, Copy, PartialEq, clap::ValueEnum)]
pub enum Typ {
    Img,
    Vid,
}

pub type Timestamp = chrono::NaiveDateTime;

#[derive(Debug)]
struct File {
    src: PathBuf,
    dst: PathBuf,
}

impl File {
    fn new(
        root: &Path,
        src: &Path,
        typ: Typ,
        img_dir: &str,
        vid_dir: &str,
        ts: Timestamp,
        hash: Hash,
        digest: &str,
    ) -> Self {
        Self {
            src: src.to_path_buf(),
            dst: dst(
                root,
                src,
                typ,
                img_dir,
                vid_dir,
                ts,
                hash.name(),
                digest,
            ),
        }
    }

    fn show(&self, dst_root: &Path) {
        println!("{:?} --> {:?}", self.src, dst_root.join(&self.dst));
    }

    #[tracing::instrument]
    fn organize(
        &self,
        dst_root: &Path,
        permanently: bool,
        force: bool,
    ) -> anyhow::Result<()> {
        tracing::info!("Organizing");
        let src = self.src.as_path();
        let dst = dst_root.join(&self.dst);
        if let Some(dst_parent) = dst.parent() {
            fs::create_dir_all(dst_parent).context(format!(
                "Failed to create parent dir: {:?}",
                dst_parent
            ))?;
        }
        let exists = dst.try_exists()?;
        if exists && src == dst {
            // XXX src should already be canonicalized.
            tracing::warn!(?src, ?dst, "Skipping. Identical src and dst.");
            return Ok(());
        }
        if exists && !force {
            tracing::warn!(
                ?dst,
                "Skipping. dst exists, but force overwrite not requested."
            );
            return Ok(());
        }
        if permanently {
            tracing::info!("Moving");
            fs::rename(src, &dst).context(format!(
                "Failed to rename file. src={:?}. dst={:?}",
                src, &dst
            ))?;
        } else {
            tracing::info!("Copying");
            fs::copy(src, &dst).context(format!(
                "Failed to copy file. src={:?}. dst={:?}",
                src, &dst
            ))?;
        }
        Ok(())
    }
}

fn auxiliary_subpath(
    root: &Path,
    path: &Path,
    ty: Typ,
    img_dir: &str,
    vid_dir: &str,
) -> Option<PathBuf> {
    use std::path::Component;

    let parent = path.parent()?;
    let root = match ty {
        Typ::Img => root.join(img_dir),
        Typ::Vid => root.join(vid_dir),
    };
    let mid_components: Vec<Component> =
        parent.strip_prefix(root).ok()?.components().collect();
    match &mid_components[..] {
        [Component::Normal(y), Component::Normal(m), Component::Normal(d), subpath @ ..]
            if are_nums(&[y, m, d]) && !subpath.is_empty() =>
        {
            Some(subpath.iter().collect())
        }
        _ => None,
    }
}

fn are_nums(strs: &[&OsStr]) -> bool {
    let count_all = strs.len();
    let count_nums = strs
        .iter()
        .filter_map(|n| n.to_str())
        .filter_map(|n| n.parse::<usize>().ok())
        .count();
    count_all == count_nums
}

#[tracing::instrument]
fn read_type(path: &Path) -> Option<Typ> {
    infer::get_from_path(path)
        .map(|matcher_type_opt| {
            tracing::debug!(?matcher_type_opt, "Read");
            matcher_type_opt.map(|typ| typ.matcher_type()).and_then(
                |matcher_type| match matcher_type {
                    infer::MatcherType::Image => Some(Typ::Img),
                    infer::MatcherType::Video => Some(Typ::Vid),
                    _ => None,
                },
            )
        })
        .map_err(|error| {
            tracing::error!(?error, "Failed");
        })
        .ok()
        .flatten()
}

#[allow(clippy::too_many_arguments)] // TODO Remove, after combining args.
#[tracing::instrument(skip_all)]
pub fn organize<'a>(
    src_root: &'a Path,
    dst_root: &'a Path,
    op: &Op,
    img_dir: &'a str,
    vid_dir: &'a str,
    ty_filter: Option<Typ>,
    force: bool,
    use_exiftool: bool,
    show_progress: bool,
    hash: Hash,
) -> anyhow::Result<()> {
    tracing::info!(?op, ?src_root, ?dst_root, "Starting");
    let src_root = src_root.canonicalize().context(format!(
        "Failed to canonicalize src path: {:?}",
        src_root
    ))?;
    if !dst_root.try_exists().context(format!(
        "Failed to check existence of dst path: {:?}",
        &dst_root
    ))? {
        tracing::info!(path = ?dst_root, "Dst dir does not exist. Creating.");
        fs::create_dir_all(dst_root)
            .context(format!("Failed to create dst dir: {:?}", dst_root))?;
    }
    let dst_root = dst_root.canonicalize().context(format!(
        "Failed to canonicalize dst path: {:?}",
        dst_root
    ))?;
    tracing::info!(?src_root, ?dst_root, "Canonicalized");
    let progress_bar = match op {
        Op::Copy | Op::Move if show_progress => {
            indicatif::ProgressBar::new(0)
        }
        _ => indicatif::ProgressBar::hidden(),
    };
    let progress_style = indicatif::ProgressStyle::with_template(
        "{bar:100.green} {pos:>7} / {len:7}",
    )?;
    progress_bar.set_style(progress_style);
    progress_bar.tick();
    FilePaths::find(&src_root)
        .par_bridge()
        .filter_map(|p| {
            progress_bar.inc_length(1);
            read_type(&p).map(|t| (p, t))
        })
        .filter_map(move |(path, ty_found)| {
            tracing::debug!(?path, ?ty_filter, ?ty_found, "Type filter");
            match ty_filter {
                Some(ty_filter) if ty_filter == ty_found => {
                    Some((path, ty_found))
                }
                None => Some((path, ty_found)),
                Some(_) => None,
            }
        })
        .filter_map(move |(path, typ)| {
            read_timestamp(&path, typ, use_exiftool)
                .ok()
                .flatten()
                .map(|timestamp| (path, typ, timestamp))
        })
        .filter_map(move |(path, typ, timestamp)| {
            hash.digest(&path)
                .ok()
                .map(|digest| (path, typ, timestamp, digest))
        })
        .map(move |(path, typ, timestamp, digest)| {
            File::new(
                &src_root, &path, typ, img_dir, vid_dir, timestamp, hash,
                &digest,
            )
        })
        .for_each(|file| {
            let result = match op {
                Op::Show => {
                    file.show(&dst_root);
                    Ok(())
                }
                Op::Copy => file.organize(&dst_root, false, force),
                Op::Move => file.organize(&dst_root, true, force),
            };
            if let Err(error) = result {
                tracing::error!(?error, ?file, "Failed to organize");
            }
            progress_bar.inc(1);
        });
    progress_bar.finish();
    tracing::info!("Finished");
    Ok(())
}

// Ref: exif::tag::d_datetime (private).
fn get_date_time_original(exif: &exif::Exif) -> Option<exif::DateTime> {
    exif.get_field(exif::Tag::DateTimeOriginal, exif::In::PRIMARY)
        .and_then(|field| match &field.value {
            exif::Value::Ascii(data) => Some(data),
            _ => None,
        })
        .and_then(|data| data.first())
        .and_then(|data| exif::DateTime::from_ascii(data).ok())
}

fn date_time_exif_to_chrono(
    dt: &exif::DateTime,
) -> Option<chrono::NaiveDateTime> {
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

fn dst(
    root: &Path,
    src: &Path,
    typ: Typ,
    img_dir: &str,
    vid_dir: &str,
    ts: Timestamp,
    hash_name: &str,
    digest: &str,
) -> PathBuf {
    use chrono::{Datelike, Timelike}; // Access timestamp fields.

    let year = format!("{:02}", ts.year());
    let month = format!("{:02}", ts.month());
    let day = format!("{:02}", ts.day());
    let hour = format!("{:02}", ts.hour());
    let minute = format!("{:02}", ts.minute());
    let second = format!("{:02}", ts.second());

    let stem = [
        [year.as_str(), month.as_str(), day.as_str()].join("-"),
        [hour, minute, second].join(":"),
        [hash_name, digest].join(":"),
    ]
    .join("--");

    let extension = src.extension().unwrap_or_default().to_ascii_lowercase();
    let name = PathBuf::from(stem).with_extension(extension);
    let typ_dir = match typ {
        Typ::Img => img_dir,
        Typ::Vid => vid_dir,
    };
    let mut dir: PathBuf = [typ_dir, &year, &month, &day].iter().collect();
    if let Some(aux) = auxiliary_subpath(root, src, typ, img_dir, vid_dir) {
        dir.push(aux);
    }
    dir.join(name)
}

#[tracing::instrument]
fn read_timestamp(
    path: &Path,
    typ: Typ,
    use_exiftool: bool,
) -> anyhow::Result<Option<Timestamp>> {
    let file = fs::File::open(path)?;
    let timestamp: Option<Timestamp> = match typ {
        Typ::Img => read_timestamp_img(&file),
        Typ::Vid => read_timestamp_vid(&file),
    }
    .or_else(|| {
        use_exiftool
            .then(|| exiftool::read_timestamp(path))
            .flatten()
    });
    tracing::debug!(?timestamp, "Finished");
    Ok(timestamp)
}

fn read_timestamp_img(file: &fs::File) -> Option<Timestamp> {
    let mut bufreader = std::io::BufReader::new(file);
    exif::Reader::new()
        .read_from_container(&mut bufreader)
        .ok()
        .and_then(|exif| {
            get_date_time_original(&exif)
                .as_ref()
                .and_then(date_time_exif_to_chrono)
        })
}

fn read_timestamp_vid(file: &fs::File) -> Option<Timestamp> {
    nom_exif::parse_metadata(file).ok().and_then(|pairs| {
        pairs
            .iter()
            .find(|(k, _)| k == "com.apple.quicktime.creationdate")
            .map(|(_, v)| v)
            .and_then(|entry| match entry {
                nom_exif::EntryValue::Time(t) => Some(t),
                _ => None,
            })
            .map(|t| t.naive_local())
    })
}

struct FilePaths {
    frontier: VecDeque<PathBuf>,
}

impl FilePaths {
    fn find(root: &Path) -> Self {
        let mut frontier = VecDeque::new();
        frontier.push_back(root.to_path_buf());
        Self { frontier }
    }
}

impl Iterator for FilePaths {
    type Item = PathBuf;

    fn next(&mut self) -> Option<Self::Item> {
        while let Some(path) = self.frontier.pop_front() {
            match fs::metadata(&path) {
                Ok(meta) if meta.is_file() => {
                    return Some(path);
                }
                Ok(meta) if meta.is_dir() => match fs::read_dir(&path) {
                    Err(error) => {
                        tracing::error!(
                            ?path,
                            ?error,
                            "Failed to read directory",
                        );
                    }
                    Ok(entries) => {
                        for entry_result in entries {
                            match entry_result {
                                Ok(entry) => {
                                    self.frontier.push_back(entry.path());
                                }
                                Err(error) => {
                                    tracing::error!(
                                        from = ?path, ?error,
                                        "Failed to read an entry",
                                    );
                                }
                            }
                        }
                    }
                },
                Ok(meta) => {
                    tracing::debug!(
                        ?path,
                        ?meta,
                        "Neither file nor directory"
                    );
                }
                Err(error) => {
                    tracing::error!(
                        from = ?path, ?error,
                        "Failed to read metadata",
                    );
                }
            }
        }
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn t_auxiliary_subpath() {
        let root = PathBuf::from("/a/b/c");
        let ty = Typ::Img;
        let img_dir = "img";
        let vid_dir = "vid";

        // Single level aux subdir:
        let path = PathBuf::from("/a/b/c/img/2009/01/07/foo/file.jpg");
        let aux = auxiliary_subpath(&root, &path, ty, img_dir, vid_dir);
        assert_eq!(Some(PathBuf::from("foo")), aux);

        // Multi level aux subdir:
        let path =
            PathBuf::from("/a/b/c/img/2009/01/07/foo/bar/baz/file.jpg");
        let aux = auxiliary_subpath(&root, &path, ty, img_dir, vid_dir);
        assert_eq!(Some(PathBuf::from("foo/bar/baz")), aux);

        // No aux subdir:
        let path = PathBuf::from("/a/b/c/img/2009/01/07/file.jpg");
        let aux = auxiliary_subpath(&root, &path, ty, img_dir, vid_dir);
        assert_eq!(None, aux);

        // Not proper date path:
        let path = PathBuf::from("/a/b/c/img/2009/01/foo/file.jpg");
        let aux = auxiliary_subpath(&root, &path, ty, img_dir, vid_dir);
        assert_eq!(None, aux);

        // Relative path:
        let root = PathBuf::from("/a/b/c");
        let path = PathBuf::from("a/b/c/img/2009/01/07/foo/file.jpg");
        let aux = auxiliary_subpath(&root, &path, ty, img_dir, vid_dir);
        assert_eq!(None, aux);

        // Relative root:
        let root = PathBuf::from("a/b/c");
        let path = PathBuf::from("a/b/c/img/2009/01/07/foo/file.jpg");
        let aux = auxiliary_subpath(&root, &path, ty, img_dir, vid_dir);
        assert_eq!(Some(PathBuf::from("foo")), aux);

        // Root mismatch:
        let root = PathBuf::from("a/b/c");
        let path = PathBuf::from("/a/b/c/img/2009/01/07/foo/file.jpg");
        let aux = auxiliary_subpath(&root, &path, ty, img_dir, vid_dir);
        assert_eq!(None, aux);

        // Root mismatch:
        let root = PathBuf::from("/meh");
        let path = PathBuf::from("/a/b/c/img/2009/01/07/foo/file.jpg");
        let aux = auxiliary_subpath(&root, &path, ty, img_dir, vid_dir);
        assert_eq!(None, aux);
    }
}
