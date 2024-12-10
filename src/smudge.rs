use crate::create_gfs_ref;
use crate::pointer::Pointer;
use anyhow::{anyhow, Result};
use gix::Repository;
use std::io::{stdout, Write};
use std::path::Path;

pub fn smudge(repo: &Repository, filepath: impl AsRef<Path>) -> Result<()> {
    let index = repo.index_or_empty()?;
    let path = filepath
        .as_ref()
        .to_str()
        .ok_or_else(|| anyhow!("Expected path to be convertable to a string slice"))?;

    let file_entry = index
        .entry_by_path(path.into())
        .ok_or_else(|| anyhow!("Expected to find the entry "))?;

    let blob = repo.find_blob(file_entry.id)?;
    let pointer: Pointer = serde_json::from_slice(&blob.data)?;
    let hash = pointer.hash();

    // get reference
    let mut reference = repo.find_reference(&create_gfs_ref(hash))?;
    let tree = reference.peel_to_tree()?;

    for entry in tree.iter() {
        let data = &entry?.object()?.data;
        stdout().write_all(data.as_slice())?;
    }

    Ok(())
}
