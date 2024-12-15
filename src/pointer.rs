use anyhow::Result;
use serde::{Deserialize, Serialize};

#[derive(Debug, PartialEq, Deserialize, Serialize)]
#[serde(tag = "version", content = "pointer")]
pub enum Pointer {
    V1 { hash: String },
}

impl From<String> for Pointer {
    fn from(value: String) -> Self {
        Self::V1 { hash: value }
    }
}

impl Pointer {
    pub fn try_to_string(&self) -> Result<String> {
        let contents = serde_json::to_string_pretty(self)?;
        Ok(contents)
    }

    pub fn hash(&self) -> &str {
        match self {
            Self::V1 { hash } => hash,
        }
    }
}
