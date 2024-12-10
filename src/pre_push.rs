use anyhow::Result;
use gix::Repository;
use std::io::{stdin, Read};

// get all the commits within the range (from stdin)
// get list of files for each commit & `.gitattributes`
// filter for big files (where they could have been added)
// read these files (from git objects directly)
// get the reference hash
// push `/refs/gfs/<hash>`, one hash at a time.
pub fn pre_push(repo: &mut Repository) -> Result<()> {
    // hash is 64 chars, surely 4 of these will be big enough.
    let mut buf = Vec::with_capacity(64 * 4);
    stdin().read_to_end(&mut buf)?;

    todo!("pre-push hook not implemented");
}
