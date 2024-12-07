mod clean;
mod map_ok_then;
mod pointer;
mod post_commit;
mod smudge;
mod splitter;

use std::path::PathBuf;

use anyhow::Result;
use bytesize::ByteSize;
use clap::Parser;
use clap_verbosity_flag::{InfoLevel, Verbosity};
use clean::Clean;
use gix::ThreadSafeRepository;
use post_commit::PostCommit;
use serde::{Deserialize, Serialize};
use smudge::Smudge;

pub const REFS_NAMESPACE: &str = "refs/gfs";

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
    Clean {
        filepath: PathBuf,
        #[arg(default_value_t = ByteSize::mb(50))]
        size: ByteSize,
    },
    Smudge(Smudge),
    PostCommit(PostCommit),
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
        let mut repo = ThreadSafeRepository::open(".")?.to_thread_local();

        match self {
            Self::Clean { filepath, size } => {
                let clean = Clean::new(filepath, size)?;

                clean.git_clean()?;
            }
            Self::Smudge(smudge) => {
                smudge.run(&repo)?;
            }
            Self::PostCommit(post_commit) => {
                PostCommit::run(&mut repo)?;
                // find all pointers that we have created/updated
                // upload all missing references to objects related to pointers
                // (blobs, trees) as their /refs/gfs/:commit
                // then upload the merge commit
            }
        }

        Ok(())
    }

    /// Returns `true` if the command is [`Clean`].
    ///
    /// [`Clean`]: Command::Clean
    #[must_use]
    pub fn is_clean(&self) -> bool {
        matches!(self, Self::Clean { .. })
    }
}

#[derive(Parser)]
pub struct Args {
    #[command(flatten)]
    pub verbosity: Verbosity<InfoLevel>,

    #[command(subcommand)]
    pub command: Command,
}
