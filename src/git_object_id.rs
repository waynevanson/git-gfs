use itertools::Itertools;
use serde::Serializer;
use std::ffi::OsStr;
use std::fmt::Display;
use std::io::{BufRead, Error as IOError, Result as IOResult};
use std::process::Stdio;
use std::{io::Write, process::Command};

#[derive(PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Clone)]
pub struct GitObjectId {
    sha: String,
}

impl GitObjectId {
    pub fn from_contents(data: impl AsRef<[u8]>) -> IOResult<Self> {
        let child = Command::new("git")
            .args(["hash-object", "-w", "--no-filters", "--stdin"])
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .spawn()?;

        // using stdin so we don't have to specify a file to read
        child
            .stdin
            .as_ref()
            .ok_or_else(|| IOError::other("Expected stdin to exist"))?
            .write_all(data.as_ref())?;

        let git_sha: String = child
            .wait_with_output()?
            .stdout
            .try_into()
            .map_err(IOError::other)?;

        // remove newlines (always at end)
        let inner = git_sha.lines().join("");

        Ok(Self { sha: inner })
    }

    pub fn size(&self) -> IOResult<u32> {
        Command::new("git")
            .args(["cat-file", "-s"])
            .arg(&self.sha)
            .output()?
            .stdout
            .lines()
            .collect::<IOResult<String>>()
            .iter()
            .join(&"")
            .parse::<u32>()
            .map_err(IOError::other)
    }
}

impl Display for GitObjectId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.serialize_str(&self.sha)
    }
}

impl AsRef<OsStr> for GitObjectId {
    fn as_ref(&self) -> &OsStr {
        self.sha.as_ref()
    }
}
