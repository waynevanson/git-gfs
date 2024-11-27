use anyhow::{anyhow, Result};
use clap::Parser;
use git2::{Config, ErrorCode, Repository, Submodule};
use std::path::PathBuf;

#[derive(Parser)]
pub struct Install {
    url: String,
    path: PathBuf,
}

// create submodule
// add config into git
impl Install {
    pub fn install(&self, repo: &Repository) -> Result<()> {
        let submodule = find_or_create_submodule(repo, self)?;

        let mut config = repo.config()?;
        create_config(&mut config)?;

        Ok(())
    }
}

fn create_config(config: &mut Config) -> Result<()> {
    config.set_str("filter.gfs.clean", "gfs clean %f")?;
    config.set_str("filter.gfs.smudge", "gfs smudge %f")?;
    config.set_bool("filter.gfs.required", true)?;

    Ok(())
}

fn find_submodule<'repo>(
    repo: &'repo Repository,
    options: &Install,
) -> Result<Option<Submodule<'repo>>> {
    let pathname = options
        .path
        .to_str()
        .ok_or_else(|| anyhow!("Expected PathBuf to be transformed to a string slice"))?;

    let submodule = allow_not_found(repo.find_submodule(pathname))?;

    Ok(submodule)
}

fn allow_not_found<T>(
    result: std::result::Result<T, git2::Error>,
) -> Result<Option<T>, git2::Error> {
    result.map(Some).or_else(|error| {
        if let ErrorCode::NotFound = error.code() {
            Ok(None)
        } else {
            Err(error)
        }
    })
}

fn create_submodule<'repo>(repo: &'repo Repository, options: &Install) -> Result<Submodule<'repo>> {
    let mut submodule = repo.submodule(&options.url.to_string(), &options.path, true)?;
    submodule.init(true)?;
    submodule.update(true, None)?;
    submodule.add_finalize()?;

    Ok(submodule)
}

fn find_or_create_submodule<'repo>(
    repo: &'repo Repository,
    options: &Install,
) -> Result<Submodule<'repo>> {
    find_submodule(repo, options).and_then(|submodule| {
        submodule
            .map(Ok)
            .unwrap_or_else(|| create_submodule(repo, options))
    })
}
