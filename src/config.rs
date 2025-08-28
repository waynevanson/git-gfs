use bytesize::ByteSize;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Deserialize, Serialize, Default)]
pub struct Config {
    pub directory: Option<PathBuf>,
    #[serde(default)]
    pub clean: CleanConfig,
}

#[derive(Deserialize, Serialize)]
pub struct CleanConfig {
    pub avg_size: ByteSize,
    pub min_size: ByteSize,
    pub max_size: ByteSize,
}

impl Default for CleanConfig {
    fn default() -> Self {
        Self {
            avg_size: ByteSize::kb(100),
            min_size: ByteSize::b(1),
            max_size: ByteSize::mb(100),
        }
    }
}
