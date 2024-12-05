use anyhow::Result;
use clap::Parser;
use gix::Repository;
use std::path::PathBuf;

#[derive(Parser)]
pub struct Smudge {
    path: PathBuf,
}

impl Smudge {
    pub fn run(repo: &Repository) -> Result<()> {
        // read the pointer file from the index?
        // find follow the tree reference to the tree
        // sort the blobs in the tree by filename
        // concatenate all parts into stdout

        Ok(())
    }
}
