use std::{io, path::Path};

#[derive(Debug, Clone, Copy, clap::ValueEnum)]
pub enum Hash {
    Sha1,
    Sha256,
    Md5,
    Crc32,
}

impl Default for Hash {
    fn default() -> Self {
        Self::Crc32
    }
}

impl Hash {
    pub fn name(&self) -> &'static str {
        match self {
            Self::Sha1 => "sha1",
            Self::Sha256 => "sha256",
            Self::Md5 => "md5",
            Self::Crc32 => "crc32",
        }
    }

    pub fn digest(&self, path: &Path) -> io::Result<String> {
        match self {
            Hash::Sha1 => digest_sha1(path),
            Hash::Sha256 => digest_sha256(path),
            Hash::Md5 => digest_md5(path),
            Hash::Crc32 => digest_crc32(path),
        }
        .map_err(|error| {
            tracing::error!(?path, algo = ?self, ?error, "Failed to hash file");
            error
        })
    }
}

fn digest_sha1<P: AsRef<Path>>(path: P) -> io::Result<String> {
    use std::io::Read;

    use sha1::{Digest, Sha1};

    let path = path.as_ref();
    let mut file = std::fs::File::open(path)?;
    let mut hash = Sha1::default();
    let mut buff = [0; 1024];
    loop {
        let n = file.read(&mut buff)?;
        if n == 0 {
            break;
        }
        hash.update(&buff[..n]);
    }
    let digest = hash.finalize();
    let hex = format!("{:x}", digest);
    Ok(hex)
}

fn digest_sha256<P: AsRef<Path>>(path: P) -> io::Result<String> {
    use std::io::Read;

    use sha2::{Digest, Sha256};

    let path = path.as_ref();
    let mut file = std::fs::File::open(path)?;
    let mut hash = Sha256::default();
    let mut buff = [0; 1024];
    loop {
        let n = file.read(&mut buff)?;
        if n == 0 {
            break;
        }
        hash.update(&buff[..n]);
    }
    let digest = hash.finalize();
    let hex = format!("{:x}", digest);
    Ok(hex)
}

fn digest_md5<P: AsRef<Path>>(path: P) -> io::Result<String> {
    use std::io::Read;

    use md5::{Digest, Md5};

    let path = path.as_ref();
    let mut file = std::fs::File::open(path)?;
    let mut hash = Md5::new();
    let mut buff = [0; 1024];
    loop {
        let n = file.read(&mut buff)?;
        if n == 0 {
            break;
        }
        hash.update(&buff[..n]);
    }
    let digest = hash.finalize();
    let hex = format!("{:x}", digest);
    Ok(hex)
}

fn digest_crc32<P: AsRef<Path>>(path: P) -> io::Result<String> {
    use std::io::Read;

    let path = path.as_ref();
    let mut file = std::fs::File::open(path)?;
    let mut hash = crc32fast::Hasher::new();
    let mut buff = [0; 1024];
    loop {
        let n = file.read(&mut buff)?;
        if n == 0 {
            break;
        }
        hash.update(&buff[..n]);
    }
    let digest = hash.finalize();
    let hex = format!("{:08x}", digest);
    Ok(hex)
}
