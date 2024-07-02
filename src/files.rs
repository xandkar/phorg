use std::{
    collections::VecDeque,
    fs,
    path::{Path, PathBuf},
};

pub fn find(path: &Path) -> Files {
    let mut frontier = VecDeque::new();
    frontier.push_back(path.to_path_buf());
    Files { frontier }
}

pub struct Files {
    frontier: VecDeque<PathBuf>,
}

impl Iterator for Files {
    type Item = PathBuf;

    fn next(&mut self) -> Option<Self::Item> {
        while let Some(path) = self.frontier.pop_front() {
            match fs::metadata(&path) {
                Ok(meta) if meta.is_file() => {
                    return Some(path);
                }
                Ok(meta) if meta.is_dir() => match fs::read_dir(&path) {
                    Err(error) => {
                        tracing::error!(?path, ?error, "Failed to read directory",);
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
                    tracing::debug!(?path, ?meta, "Neither file nor directory");
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
