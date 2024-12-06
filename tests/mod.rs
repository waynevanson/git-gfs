// copy a fixture to a tmp dir
// rename `.git_` to `.git`
//
// There's a part of me that wants to add the setup files into the test itself
// so that the test contains everything, but maybe it will make tests too big.
//
// I think I should use `git` so we know it works with git.
//
// We could keep a list of fixtures for .git dirs in fixture/* and then
// cp + paste from git.

use anyhow::Result;
use copy_dir::copy_dir;
use std::path::{Path, PathBuf};
use temp_dir::TempDir;

fn setup(fixture: impl AsRef<Path>) -> Result<TempDir> {
    let target = TempDir::new()?;

    let source = PathBuf::new().join("fixture").join(fixture);
    copy_dir(&source, &target)?;

    let gitfile = target.path().join(".git.file");
    let git = target.path().join(".git");
    std::fs::rename(gitfile, git)?;

    Ok(target)
}
