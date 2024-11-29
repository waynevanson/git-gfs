use crate::splitter::Splitter;
use anyhow::{anyhow, Result};
use bytesize::ByteSize;
use clap::Parser;
use gix::Repository;
use std::{fs::File, io::copy, path::PathBuf};

#[derive(Parser)]
pub struct Clean {
    filepath: PathBuf,
}

impl Clean {
    /// Splits a file into pieces and places them in `.git/parts/:filepath.part.[a-z]{3}`
    ///
    /// # Errors
    ///
    /// 1. Bare repository
    /// 1. When the `.git` has no parent.
    pub fn run(&self, repo: &Repository) -> Result<()> {
        if repo.is_bare() {
            return Err(anyhow!("Expected repository not to be bare"));
        }

        let git = repo.path();
        let root = git
            .parent()
            .ok_or_else(|| anyhow!("Expected .git directory of repository to have a parent"))?;

        let mut reader = File::open(root.join(&self.filepath))?;

        let size = ByteSize::mb(50).as_u64();

        let mut path = git.join("parts").join(&self.filepath);
        path.push(".part."); // todo: will this be part of the file or under a dir?
        let mut writer = Splitter::new(path, size, 3);

        copy(&mut reader, &mut writer)?;

        Ok(())
    }
}
