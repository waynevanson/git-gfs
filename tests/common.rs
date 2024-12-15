use anyhow::{anyhow, bail, Result};
use git_file_storage::SealedOutput;
use std::{
    fs::{create_dir_all, File},
    io::Write,
    path::Path,
    process::Command,
};

pub fn create_files(
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

pub fn git_commit_add_all_files(tmp: impl AsRef<Path>) -> Result<()> {
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

pub fn git_init(tmp: impl AsRef<Path>) -> Result<()> {
    Command::new("git")
        .current_dir(tmp)
        .args(["init"])
        .output()?
        .exit_ok_or_stderror()?;

    Ok(())
}
