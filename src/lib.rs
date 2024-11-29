mod clean;
mod install;
mod splitter;
mod track;

use anyhow::Result;
use clap::Parser;
use clap_verbosity_flag::{InfoLevel, Verbosity};
use clean::Clean;
use gix::{Repository, ThreadSafeRepository};
use install::Install;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use track::Track;

#[derive(Deserialize, Serialize)]
#[serde(rename = r#"filter "gfs""#)]
pub struct FilterGfs {
    clean: String,
    smudge: String,
    required: bool,
}

#[derive(Deserialize, Serialize, Default)]
pub struct GitAttribute {
    #[serde(rename = r#"filter "gfs""#)]
    filter_gfs: FilterGfs,
}

impl Default for FilterGfs {
    fn default() -> Self {
        Self {
            clean: "gfs clean".to_owned(),
            smudge: "gfs smudge".to_owned(),
            required: true,
        }
    }
}

#[derive(Parser)]
pub enum Command {
    // create submodule in repo
    // add gitattributes filter to .git/config or ~/.gitconfig
    Install(Install),
    // add git attribute with filter for file pattern.s
    Track(Track),
    // split.
    Clean(Clean),
    // join
    // - Check to see if parts are stored in commits by checking location against contents of file.
    // - Move them to `/git/parts`. How should I hash this so there's no conflicts?
    // - Join and concat.
    Smudge { filepath: PathBuf },
    // Ensure pack is smaller than x
    PrePush { size: usize },
    PreCommit(PreCommit),
}

#[derive(Parser)]
pub struct PreCommit;

impl PreCommit {
    pub fn run(repo: &Repository) -> Result<()> {
        // commit each part into refs/split/<commit-hash> from the previous commit (without the file)
        // remove the previous part? Maybe we need it so the file is actually added. tow parents, makes sense.
        // merge the last into this.

        Ok(())
    }
}

// When a user pushes and git hooks are on, it should automatically
// automatically push the other commit.
impl Command {
    /// Runs the main command.
    ///
    /// # Errors
    /// 1. Opening a repository.
    /// 2. Running sub commands
    pub fn run(self) -> Result<()> {
        let repo = ThreadSafeRepository::open(".")?.to_thread_local();

        match self {
            Self::Install(_) => {
                unimplemented!();
            }
            Self::Clean(clean) => {
                clean.run(&repo)?;

                unimplemented!();
            }
            Self::Smudge { .. } => {
                // merge into files.
                unimplemented!();
            }
            Self::Track(_) => {
                unimplemented!();
            }

            Self::PreCommit(_) => {
                unimplemented!();
            }
            Self::PrePush { .. } => {
                unimplemented!();
            }
        }
    }
}

#[derive(Parser)]
pub struct Args {
    #[command(flatten)]
    pub verbosity: Verbosity<InfoLevel>,

    #[command(subcommand)]
    pub command: Command,
}
