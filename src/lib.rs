mod clean;
mod map_ok_then;
mod pointer;
mod post_commit;
mod smudge;
mod splitter;

use anyhow::Result;
use clap::Parser;
use clap_verbosity_flag::{InfoLevel, Verbosity};
use clean::Clean;
use gix::ThreadSafeRepository;
use serde::{Deserialize, Serialize};
use smudge::Smudge;

pub const REFS_NAMESPACE: &str = "/refs/gfs";

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
    // add git attribute with filter for file pattern.s
    // split.
    Clean(Clean),
    // join
    // - Check to see if parts are stored in commits by checking location against contents of file.
    // - Move them to `/git/parts`. How should I hash this so there's no conflicts?
    // - Join and concat.
    // read the pointer and get all the refs.
    Smudge(Smudge),
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
            Self::Clean(clean) => {
                clean.run(&repo)?;
            }
            Self::Smudge(smudge) => {
                smudge.run(&repo)?;
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
