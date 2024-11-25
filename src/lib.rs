mod install;
mod track;

use anyhow::Result;
use clap::Parser;
use clap_verbosity_flag::{InfoLevel, Verbosity};
use git2::Repository;
use install::Install;
use serde::{Deserialize, Serialize};
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
        let repo = Repository::open(".")?;

        match self {
            Self::Install(install) => {
                unimplemented!();
            }
            Self::Clean {} => {
                // split into submodule.
                unimplemented!();
            }
            Self::Smudge {} => {
                // merge into files.
                unimplemented!();
            }
            Self::Track(track) => {
                track.track(&repo)?;
            }
            Self::PrePush { size } => {
                unimplemented!();
            }
        }

        Ok(())
    }
}

#[derive(Parser)]
pub struct Args {
    #[command(flatten)]
    pub verbosity: Verbosity<InfoLevel>,

    #[command(subcommand)]
    pub command: Command,
}
