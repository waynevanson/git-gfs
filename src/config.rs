use bytesize::ByteSize;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

// on cehck in, save a pointer and add parts to .gfs/blobs/[a-z0-9]{2}/[rest]
// on checkout, re
#[derive(Deserialize, Serialize, Default)]
pub struct Config {
    pub directory: Option<PathBuf>,
    #[serde(default)]
    pub clean: CleanConfig,
    #[serde(default)]
    pub pre_push: PrePushConfig,
}

#[derive(Deserialize, Serialize)]
pub struct CleanConfig {
    pub min_size: ByteSize,
    pub max_size: ByteSize,
    pub avg_size: ByteSize,
}

impl Default for CleanConfig {
    fn default() -> Self {
        Self {
            min_size: ByteSize(1_024),
            max_size: ByteSize(1_024 * 32),
            avg_size: ByteSize(1_024 * 8),
        }
    }
}

#[derive(Deserialize, Serialize, Default)]
pub struct PrePushConfig {
    pub limit: Limit,
}

#[derive(Deserialize, Serialize)]
pub enum Limit {
    Default(ByteSize),
}

impl Default for Limit {
    fn default() -> Self {
        Self::Default(ByteSize::mb(500))
    }
}
