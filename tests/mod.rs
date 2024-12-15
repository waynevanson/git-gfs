// States to control: file structure, git changes over time.
// I kind of want full control in code. Maybe including git times and commits etc.
// A file list of directories, maybe some

#![feature(exit_status_error)]

use anyhow::{anyhow, bail, Result};
use git_file_storage::Pointer;
use gix::bstr::ByteSlice;
use gix::ThreadSafeRepository;
use gix::{bstr::BStr, ObjectId};
use itertools::Itertools;
use std::fmt::format;
use std::process::Output;
use std::{
    fs::{create_dir_all, read_to_string, File},
    io::Write,
    path::Path,
    process::Command,
};
use tempdir::TempDir;

fn create_files(
    tmp: impl AsRef<Path>,
    files: impl IntoIterator<Item = (impl AsRef<str>, impl AsRef<str>)>,
) -> Result<()> {
    for (path, contents) in files {
        if path.as_ref().is_empty() {
            bail!("Expected path to be non-empty")
        };

        let full_path = tmp.as_ref().join(Path::new(path.as_ref()));

        let dir = full_path
            .parent()
            .ok_or_else(|| anyhow!("Expected path to have parent"))?;

        create_dir_all(dir)?;

        File::create(&full_path)?.write_all(contents.as_ref().as_bytes())?;
    }

    Ok(())
}

trait SealedOutput {
    fn exit_ok_or_stderror(self) -> Result<()>;
}

impl SealedOutput for Output {
    fn exit_ok_or_stderror(self) -> Result<()> {
        if !self.status.success() {
            let str = self.stderr.to_str()?.to_owned();
            bail!(str);
        }

        Ok(())
    }
}

fn git_commit_add_all_files(tmp: impl AsRef<Path>) -> Result<()> {
    Command::new("git")
        .current_dir(&tmp)
        .args(["add", "."])
        .output()?
        .exit_ok_or_stderror()?;

    Command::new("git")
        .current_dir(&tmp)
        .args(["commit", "--allow-empty-message", "-m", ""])
        .output()?
        .exit_ok_or_stderror()?;

    Ok(())
}

fn git_init(tmp: impl AsRef<Path>) -> Result<()> {
    Command::new("git")
        .current_dir(tmp)
        .args(["init"])
        .output()?
        .exit_ok_or_stderror()?;

    Ok(())
}

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
