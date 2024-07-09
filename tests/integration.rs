use std::path::{Path, PathBuf};

use tempfile::tempdir;

#[test]
fn from_empty_dst() {
    let src = PathBuf::from("tests/data/src");
    let dst = tempdir().unwrap();
    let dst = dst.path();
    assert!(file_paths_sorted(dst).is_empty());
    // TODO Call as shell command.
    phorg::files::organize(
        &src,
        dst,
        &phorg::files::Op::Copy,
        "img",
        "vid",
        None,
        false,
        false,
        false,
        phorg::hash::Hash::Crc32,
    )
    .unwrap();
    assert_eq!(
        &file_paths_sorted(&src),
        &vec![src.join("bar.jpg"), src.join("foo.jpg"), src.join("make"),]
    );
    assert_eq!(
        &file_paths_sorted(dst),
        &vec![
            dst.join(
                "img/2000/12/27/2000-12-27--06:47:01--crc32:383ba95e.jpg"
            ),
            dst.join(
                "img/2010/01/31/2010-01-31--17:35:49--crc32:c653b4f3.jpg"
            ),
        ]
    );
    // TODO Assert file contents did not change.
}

fn file_paths_sorted(root: &Path) -> Vec<PathBuf> {
    let mut paths: Vec<PathBuf> =
        phorg::files::FilePaths::find(&root).collect();
    paths.sort();
    paths
}
