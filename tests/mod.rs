//! Since we're testing `.git`, let's create a file
//! We can add it to the git tree as a ref?

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
