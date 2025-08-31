use std::path::Path;

use sha1::{Digest, Sha1};

#[derive(PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Clone)]
pub struct ContentSha {
    pub inner: String,
}

impl ContentSha {
    pub fn from_contents(data: impl AsRef<[u8]>) -> Self {
        let sha = Sha1::digest(data);
        let inner: String = format!("{:x}", sha);
        Self { inner }
    }
}

impl AsRef<Path> for ContentSha {
    fn as_ref(&self) -> &Path {
        self.inner.as_ref()
    }
}
