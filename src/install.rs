use anyhow::Result;
use clap::Parser;
use gix::Repository;

#[derive(Parser)]
pub struct Install;

// create submodule
// add config into git
impl Install {
    /// Idempotently install git config and git hooks.
    pub fn install(repo: &Repository) -> Result<()> {
        // add config
        hooks::add_to_repository(repo)?;

        Ok(())
    }
}

mod hooks {
    use anyhow::{anyhow, Result};
    use gix::Repository;
    use std::{fs::File, io::Write};

    const POST_COMMIT_SCRIPT: &str = r"
        #!/bin/sh

        gfs post-commit %f
    ";

    const PRE_PUSH_SCRIPT: &str = r"
        #!/bin/sh

        gfs pre-push %f
    ";

    /// Adds git hooks to a repository.
    ///
    /// # Errors
    ///
    /// 1. Bare repository.
    /// 2. Writing to files.
    pub fn add_to_repository(repo: &Repository) -> Result<()> {
        if repo.is_bare() {
            let message = "gfs install only works in non bare repositories, as we need the .git folder to add the hooks into";
            return Err(anyhow!(message));
        }

        let hooks_dir = repo.path().join("hooks");

        let mut pre_commit_file = File::create_new(hooks_dir.join("post-commit"))?;
        pre_commit_file.write_all(POST_COMMIT_SCRIPT.trim().as_ref())?;

        let mut pre_push_file = File::create_new(hooks_dir.join("post-commit"))?;
        pre_push_file.write_all(PRE_PUSH_SCRIPT.trim().as_ref())?;

        Ok(())
    }
}
