use std::{io, path::Path};

#[derive(Debug, Clone, Copy, clap::ValueEnum)]
pub enum Hash {
    Sha256,
    Md5,
}

impl Default for Hash {
    fn default() -> Self {
        Self::Md5
    }
}

impl Hash {
    pub fn name(&self) -> &'static str {
        match self {
            Self::Sha256 => "sha256",
            Self::Md5 => "md5",
        }
    }

    pub fn digest(&self, path: &Path) -> io::Result<String> {
        match self {
            Hash::Sha256 => digest_sha256(path),
            Hash::Md5 => digest_md5(path),
        }
        .map_err(|error| {
            tracing::error!(?path, algo = ?self, ?error, "Failed to hash file");
            error
        })
    }
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
