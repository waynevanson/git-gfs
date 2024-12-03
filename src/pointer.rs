use anyhow::Result;
use serde::Serialize;

#[derive(Serialize, Default)]
pub enum HashType {
    #[allow(dead_code)]
    SHA1,
    #[default]
    SHA256,
}

#[derive(Serialize, Default)]
pub enum Version {
    #[default]
    One,
}

#[derive(Serialize)]
pub struct Pointer {
    hash_function: HashType,
    hash: String,
    version: Version,
}

impl Pointer {
    pub fn from_sha(hash_function: HashType, hash: String) -> Self {
        Self {
            hash,
            hash_function,
            version: Version::default(),
        }
    }

    pub fn try_to_string(&self) -> Result<String> {
        let contents = toml::to_string_pretty(&self)?;
        Ok(contents)
    }
}
