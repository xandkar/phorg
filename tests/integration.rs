use std::{
    fs::{self, File},
    io::{self, Read},
    path::{Path, PathBuf},
};

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

    let foo_before = "foo.jpg";
    let foo_after = "img/2000/12/27/2000-12-27--06:47:01--crc32:383ba95e.jpg";
    let bar_before = "bar.jpg";
    let bar_after = "img/2010/01/31/2010-01-31--17:35:49--crc32:c653b4f3.jpg";

    assert_eq!(
        &file_paths_sorted(&src),
        &vec![src.join(bar_before), src.join(foo_before), src.join("make"),]
    );
    assert_eq!(
        &file_paths_sorted(dst),
        &vec![dst.join(foo_after), dst.join(bar_after),]
    );
    assert!(files_eq(src.join(foo_before), dst.join(foo_after)).unwrap());
    assert!(files_eq(src.join(bar_before), dst.join(bar_after)).unwrap());
}

fn file_paths_sorted(root: &Path) -> Vec<PathBuf> {
    let mut paths: Vec<PathBuf> =
        phorg::files::FilePaths::find(&root).collect();
    paths.sort();
    paths
}

fn files_eq<P: AsRef<Path>>(path_a: P, path_b: P) -> io::Result<bool> {
    const CHUNK_SIZE: usize = 8;

    let path_a = path_a.as_ref();
    let path_b = path_b.as_ref();

    let size_a = fs::metadata(path_a)?.len();
    let size_b = fs::metadata(path_b)?.len();
    if size_a != size_b {
        return Ok(false);
    }

    let mut file_a = File::open(path_a)?;
    let mut file_b = File::open(path_b)?;
    let mut buff_a = [0; CHUNK_SIZE];
    let mut buff_b = [0; CHUNK_SIZE];
    loop {
        let n_a = file_a.read(&mut buff_a)?;
        let n_b = file_b.read(&mut buff_b)?;

        let chunk_a = &buff_a[..n_a];
        let chunk_b = &buff_b[..n_b];

        if n_a != n_b {
            return Ok(false);
        }
        if chunk_a != chunk_b {
            return Ok(false);
        }
        if n_a == 0 {
            assert_eq!(n_b, 0);
            return Ok(true);
        }
    }
}
