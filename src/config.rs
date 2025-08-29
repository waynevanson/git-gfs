use bytesize::ByteSize;
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize)]
pub struct Config {
    #[serde(default = "avg_size")]
    pub avg_size: ByteSize,
    #[serde(default = "min_size")]
    pub min_size: ByteSize,
    #[serde(default = "max_size")]
    pub max_size: ByteSize,
}

const fn avg_size() -> ByteSize {
    ByteSize::kb(100)
}

const fn min_size() -> ByteSize {
    ByteSize::b(1)
}

const fn max_size() -> ByteSize {
    ByteSize::mb(1)
}

impl Default for Config {
    fn default() -> Self {
        Self {
            avg_size: avg_size(),
            min_size: min_size(),
            max_size: max_size(),
        }
    }
}
