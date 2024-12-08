use crate::create_gfs_ref;
use crate::{map_ok_then::MapOkThen, pointer::Pointer};
use anyhow::{anyhow, Result};
use core::str;
use gix::{Repository, ThreadSafeRepository};
use itertools::Itertools;
use std::fs::File;
use std::io::Write;
use std::path::PathBuf;

pub struct Smudge {
    path: PathBuf,
    repo: Repository,
}

// check file contents in index.
// concatente parts
impl Smudge {
    pub fn new(filepath: PathBuf) -> Result<Self> {
        let repo = ThreadSafeRepository::open(".")?.to_thread_local();
        let smudge = Self {
            path: filepath,
            repo,
        };
        Ok(smudge)
    }

    pub fn git_smudge(&self) -> anyhow::Result<()> {
        let index = self.repo.index_or_empty()?;
        let path = self.repo.path().to_str().ok_or_else(|| anyhow!("lol"))?;
        let file = index
            .entry_by_path(path.into())
            .ok_or_else(|| anyhow!("Wowzers"))?;

        let blob = self.repo.find_blob(file.id)?;
        let string = str::from_utf8(&blob.data)?;
        let pointer: Pointer = serde_json::from_str(string)?;

        let hash = pointer.hash();

        // get reference
        let mut reference = self.repo.find_reference(&create_gfs_ref(hash))?;
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
