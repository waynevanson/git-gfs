use crate::REFS_NAMESPACE;
use crate::{map_ok_then::MapOkThen, pointer::Pointer};
use anyhow::{anyhow, Result};
use clap::Parser;
use core::str;
use gix::Repository;
use itertools::Itertools;
use std::fs::File;
use std::io::Write;
use std::path::PathBuf;

#[derive(Parser)]
pub struct Smudge {
    path: PathBuf,
}

// check file contents in index.
// concatente parts
impl Smudge {
    pub fn run(&self, repo: &Repository) -> anyhow::Result<()> {
        let index = repo.index_or_empty()?;
        let path = repo.path().to_str().ok_or_else(|| anyhow!("lol"))?;
        let file = index
            .entry_by_path(path.into())
            .ok_or_else(|| anyhow!("Wowzers"))?;

        let blob = repo.find_blob(file.id)?;
        let string = str::from_utf8(&blob.data)?;
        let pointer: Pointer = toml::from_str(string)?;

        let hash = pointer.hash;

        // get reference
        let mut reference = repo.find_reference(&format!("{REFS_NAMESPACE}/{hash}"))?;
        let tree = reference.peel_to_tree()?;
        let datas: Vec<_> = tree
            .iter()
            .map(|result| -> Result<_> { Ok(result?) })
            .map_ok_then(|entry_ref| {
                let object = entry_ref.object()?;
                let data = object.data.clone();
                Ok(data)
            })
            .flatten_ok()
            .try_collect()?;

        let mut file = File::create(&self.path)?;

        file.write_all(&datas)?;

        Ok(())
    }
}
