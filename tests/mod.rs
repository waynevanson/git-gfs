// States to control: file structure, git changes over time.
// I kind of want full control in code. Maybe including git times and commits etc.
// A file list of directories, maybe some

use anyhow::{anyhow, bail, Result};
use std::{
    fs::{create_dir_all, File},
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

fn git_commit_add_all_files(tmp: impl AsRef<Path>) -> Result<()> {
    Command::new("git")
        .current_dir(&tmp)
        .args(["add", "."])
        .output()?;

    Command::new("git")
        .current_dir(&tmp)
        .args(["commit", "--allow-empty-message"])
        .output()?;

    Ok(())
}

fn git_init(tmp: impl AsRef<Path>) -> Result<()> {
    Command::new("git")
        .current_dir(tmp)
        .args(["init"])
        .output()?;

    Ok(())
}

fn setup(tmp: impl AsRef<Path>) -> Result<()> {
    let files = [("src/file.one.txt", "World!")];
    create_files(&tmp, files)?;
    git_commit_add_all_files(&tmp)?;

    Ok(())
}

#[test]
fn should_clean_a_file() -> Result<()> {
    let tmp = TempDir::new("")?;

    git_init(&tmp)?;

    let files = [
        (".gitattributes", "*.txt filter=gfs -text"),
        (".git/config", r#"[filter "gfs"]\nclean=git-gfs clean %f"#),
    ];
    create_files(&tmp, files)?;
    git_commit_add_all_files(&tmp)?;

    // assert there are no refs

    let files = [("src/file.one.txt", "Hello")];
    create_files(&tmp, files)?;
    git_commit_add_all_files(&tmp)?;

    // create file
    // git add
    // git commit
    // expect file to be a pointer.
    // expect all the refs to exist.

    Ok(())
}
