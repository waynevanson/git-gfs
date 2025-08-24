use anyhow::{anyhow, Result};
use gix::Repository;
use itertools::Itertools;
use std::{io::stdin, process::Command};

// get all the commits within the range (from stdin)
// get list of files for each commit & `.gitattributes`
// filter for big files (where they could have been added)
// read these files (from git objects directly)
// get the reference hash
// push `/refs/gfs/<hash>`, one hash at a time.
pub fn pre_push(repo: &mut Repository, url: String) -> Result<()> {
    let mut wants = String::new();
    let mut haves = String::new();

    for line in stdin().lines() {
        let line = line?;

        let (_local_ref, local_sha1, _remote_ref, remote_sha1) = line
            .split(" ")
            .collect_tuple::<(&str, &str, &str, &str)>()
            .ok_or_else(|| anyhow!("Expected to be able to split the line into 4 segments"))?;

        let cmd = Command::new("git").args(["rev-list", local_sha1, "--not"]);
    }

    // check what was ackowledged

    Ok(())
}
