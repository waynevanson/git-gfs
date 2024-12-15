use serde::{Deserialize, Serialize};

/// Contains a hash that points to the reference hash.
///
#[derive(Debug, PartialEq, Deserialize, Serialize)]
#[serde(tag = "version", content = "pointer")]
pub enum Pointer {
    V1 { hash: String },
}

impl TryFrom<&str> for Pointer {
    type Error = serde_json::Error;

    fn try_from(value: &str) -> std::result::Result<Self, Self::Error> {
        serde_json::from_str(value)
    }
}

impl TryFrom<&Pointer> for String {
    type Error = serde_json::Error;

    fn try_from(value: &Pointer) -> std::result::Result<Self, Self::Error> {
        serde_json::to_string_pretty(value)
    }
}

impl Pointer {
    pub fn from_hash(hash: impl AsRef<str>) -> Self {
        Self::V1 {
            hash: hash.as_ref().to_owned(),
        }
    }

    pub fn hash(&self) -> &str {
        match self {
            Self::V1 { hash } => hash,
        }
    }
}
