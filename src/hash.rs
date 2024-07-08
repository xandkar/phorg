use std::{
    io::{self, Read},
    path::Path,
};

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
        let mut file = std::fs::File::open(path)?;
        match self {
            Hash::Sha1 => digest_sha1(&mut file),
            Hash::Sha256 => digest_sha256(&mut file),
            Hash::Md5 => digest_md5(&mut file),
            Hash::Crc32 => digest_crc32(&mut file),
        }
        .map_err(|error| {
            tracing::error!(?path, algo = ?self, ?error, "Failed to hash file");
            error
        })
    }
}

fn digest_sha1<R: Read>(data: &mut R) -> io::Result<String> {
    use sha1::{Digest, Sha1};

    let mut hash = Sha1::new();
    let mut buff = [0; 1024];
    loop {
        let n = data.read(&mut buff)?;
        if n == 0 {
            break;
        }
        hash.update(&buff[..n]);
    }
    let digest = hash.finalize();
    let hex = format!("{:x}", digest);
    Ok(hex)
}

fn digest_sha256<R: Read>(data: &mut R) -> io::Result<String> {
    use sha2::{Digest, Sha256};

    let mut hash = Sha256::new();
    let mut buff = [0; 1024];
    loop {
        let n = data.read(&mut buff)?;
        if n == 0 {
            break;
        }
        hash.update(&buff[..n]);
    }
    let digest = hash.finalize();
    let hex = format!("{:x}", digest);
    Ok(hex)
}

fn digest_md5<R: Read>(data: &mut R) -> io::Result<String> {
    use md5::{Digest, Md5};

    let mut hash = Md5::new();
    let mut buff = [0; 1024];
    loop {
        let n = data.read(&mut buff)?;
        if n == 0 {
            break;
        }
        hash.update(&buff[..n]);
    }
    let digest = hash.finalize();
    let hex = format!("{:x}", digest);
    Ok(hex)
}

fn digest_crc32<R: Read>(data: &mut R) -> io::Result<String> {
    let mut hash = crc32fast::Hasher::new();
    let mut buff = [0; 1024];
    loop {
        let n = data.read(&mut buff)?;
        if n == 0 {
            break;
        }
        hash.update(&buff[..n]);
    }
    let digest = hash.finalize();
    let hex = format!("{:08x}", digest);
    Ok(hex)
}
