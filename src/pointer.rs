use anyhow::Result;
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Default)]
pub enum HashType {
    #[allow(dead_code)]
    SHA1,
    #[default]
    SHA256,
}

#[derive(Deserialize, Serialize, Default)]
pub enum Version {
    #[default]
    One,
}

#[derive(Deserialize, Serialize)]
pub struct Pointer {
    pub hash_function: HashType,
    pub hash: String,
    pub version: Version,
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
