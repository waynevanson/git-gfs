mod common;

use anyhow::{anyhow, Result};
use common::{create_files, git_commit_add_all_files, git_init};
use git_file_storage::Pointer;
use gix::bstr::ByteSlice;
use gix::ThreadSafeRepository;
use std::fs::read_to_string;
use tempdir::TempDir;

#[test]
fn should_clean_a_file() -> Result<()> {
    let tmp = TempDir::new("")?;
    println!("{tmp:?}");

    git_init(&tmp)?;

    let files = [
        (".gitattributes", "*.txt filter=gfs -text"),
        (
            ".git/config",
            "[filter \"gfs\"]\nclean=git-gfs clean %f\nrequired=true",
        ),
    ];
    create_files(&tmp, files)?;
    git_commit_add_all_files(&tmp)?;

    // assert there are no refs

    let files = [("src/file.one.txt", "Hello")];
    create_files(&tmp, files)?;
    git_commit_add_all_files(&tmp)?;

    let path = tmp.path().join(files[0].0);
    let contents = read_to_string(&path)?;

    // working directory contains same contents
    assert_eq!(contents, files[0].1);

    let repo = ThreadSafeRepository::open(tmp.as_ref().to_path_buf())?.to_thread_local();
    let index = repo.index()?;

    let entry = index
        .entry_by_path(files[0].0.into())
        .ok_or_else(|| anyhow!("Expected to find an entry by path: \"{}\"", files[0].0))?;
    let contents = &repo.find_object(entry.id)?.data;
    let pointer: Pointer = serde_json::from_slice(contents)?;
    let refname = format!("refs/gfs/{}", pointer.hash());

    let mut parts_reference = repo.find_reference(&refname)?;
    let parts_tree = parts_reference.peel_to_tree()?;
    let parts = parts_tree.traverse().breadthfirst.files()?;

    assert_eq!(parts.len(), 1);

    let part = repo.find_blob(parts[0].oid)?;
    assert_eq!(part.data.to_str()?, files[0].1);

    // expect file to be a pointer.
    // expect all the refs to exist.

    Ok(())
}
