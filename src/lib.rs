use anyhow::{anyhow, Result};
use clap::Parser;
use clap_verbosity_flag::{InfoLevel, Verbosity};
use git2::{ErrorCode, Repository, Submodule};
use git_attributes::{
    parse::{Kind, Lines},
    StateRef,
};
use itertools::Itertools;
use serde::{Deserialize, Serialize};
use std::{
    fmt::Debug,
    fs::{read, read_to_string},
    path::PathBuf,
};
use toml::Value;
use url::Url;

#[derive(Deserialize, Serialize)]
#[serde(rename = r#"filter "gfs"#)]
pub struct FilterGfs {
    clean: String,
    smudge: String,
    required: bool,
}

impl FilterGfs {
    pub fn new() -> Self {
        Self {
            clean: "gfs clean".to_owned(),
            smudge: "gfs smudge".to_owned(),
            required: true,
        }
    }

    pub fn toml_string() {}
}

#[derive(Parser)]
pub struct InstallOptions {
    url: Url,
    path: PathBuf,
}

#[derive(Parser)]
pub struct TrackOptions {
    pattern: String,
}

#[derive(Parser)]
pub enum Command {
    // create submodule in repo
    // add gitattributes filter to .git/config or ~/.gitconfig
    Install(InstallOptions),
    // add git attribute with filter for file pattern.
    Track(TrackOptions),
    // archive?, compress, split.
    Clean {},
    // join, decompress, unarchive?
    Smudge {},
    // Ensure pack is smaller than x
    PrePush { size: usize },
}

// When a user pushes and git hooks are on, it should automatically
// automatically push the other commit.
impl Command {
    pub fn call(self) -> Result<()> {
        use Command::*;
        let repo = Repository::open(".")?;

        match self {
            Install(options) => {
                let submodule = find_or_create_submodule(&repo, &options)?;
            }
            Clean {} => {
                // split into submodule.
            }
            Smudge {} => {
                // merge into files.
            }
            Self::Track(options) => {
                let attributes_path = get_attributes_path(&repo)?;

                // todo: if not exists, create.
                let contents = read(attributes_path)?;
                let attributes = git_attributes::parse(contents.as_slice());

                let attrs = attributes_has_gfs(attributes, &options);

                // does attribute we're about to create exist?
                // if so, skip.
                // If not, append.
            }
            _ => unimplemented!(),
        }

        Ok(())
    }
}

// todo: throw error at any point?
fn attributes_has_gfs(attributes: Lines<'_>, options: &TrackOptions) -> Result<bool> {
    let bool = attributes
        .filter_map_ok(|(kind, mut iter, _)| {
            let is_current_pattern = if let Kind::Pattern(pattern) = kind {
                pattern.to_string() == options.pattern
            } else {
                false
            };

            let is_filter_gfs = iter.any(|result| {
                if let Ok(assignment) = result {
                    let is_filter = assignment.name.as_str() == "filter";
                    let is_gfs = if let StateRef::Value(bstr) = assignment.state {
                        bstr.to_string() == options.pattern
                    } else {
                        false
                    };

                    is_filter && is_gfs
                } else {
                    false
                }
            });

            Some(is_current_pattern && is_filter_gfs)
        })
        .any(|result| matches!(result, Ok(_)));

    Ok(bool)
}

fn get_attributes_path(repo: &Repository) -> Result<PathBuf> {
    let attributes_path = repo
        .path()
        .parent()
        .ok_or_else(|| anyhow!("Expected the repository to have a parent path"))?
        .join(".gitattributes");

    Ok(attributes_path)
}

fn find_submodule<'repo>(
    repo: &'repo Repository,
    options: &InstallOptions,
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

fn create_submodule<'repo>(
    repo: &'repo Repository,
    options: &InstallOptions,
) -> Result<Submodule<'repo>> {
    let mut submodule = repo.submodule(&options.url.to_string(), &options.path, true)?;

    submodule.clone(None)?;
    submodule.add_finalize()?;

    Ok(submodule)
}

fn find_or_create_submodule<'repo>(
    repo: &'repo Repository,
    options: &InstallOptions,
) -> Result<Submodule<'repo>> {
    find_submodule(repo, options).and_then(|submodule| {
        submodule
            .map(Ok)
            .unwrap_or_else(|| create_submodule(repo, options))
    })
}

#[derive(Parser)]
pub struct Args {
    #[command(flatten)]
    verbosity: Verbosity<InfoLevel>,

    #[command(subcommand)]
    command: Command,
}
