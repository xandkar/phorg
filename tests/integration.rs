use std::{
    fs::{self, File},
    io::{self, Read},
    path::{Path, PathBuf},
    process::Command,
};

use assert_cmd::{assert::OutputAssertExt, cargo::CommandCargoExt};
use tempfile::tempdir;

#[test]
fn from_empty_dst() {
    let exe = env!("CARGO_PKG_NAME");
    let src = PathBuf::from("tests/data/src");
    let dst = tempdir().unwrap();
    let dst = dst.path();
    assert!(file_paths_sorted(dst).is_empty());

    let mut cmd = Command::cargo_bin(exe).unwrap();
    cmd.arg(&src).arg(&dst).arg("copy");
    cmd.assert().success();

    let foo_src = "foo.jpg";
    let foo_dst = "img/2000/12/27/2000-12-27--06:47:01--crc32:46c1273c.jpg";
    let bar_src = "bar.jpg";
    let bar_dst = "img/2010/01/31/2010-01-31--17:35:49--crc32:13c81260.jpg";
    let foo_src_path = src.join(foo_src);
    let bar_src_path = src.join(bar_src);
    let foo_dst_path = dst.join(foo_dst);
    let bar_dst_path = dst.join(bar_dst);

    assert_eq!(
        &file_paths_sorted(&src).iter().collect::<Vec<&PathBuf>>(),
        &vec![&bar_src_path, &foo_src_path, &src.join("make")]
    );
    assert_eq!(
        &file_paths_sorted(&dst).iter().collect::<Vec<&PathBuf>>(),
        &vec![&foo_dst_path, &bar_dst_path][..]
    );
    assert!(files_eq(foo_src_path, foo_dst_path).unwrap());
    assert!(files_eq(bar_src_path, bar_dst_path).unwrap());
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
