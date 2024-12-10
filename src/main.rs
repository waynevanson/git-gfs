use anyhow::Result;
use bytesize::ByteSize;
use clap::Parser;
use clap_verbosity_flag::{InfoLevel, Verbosity};
use git_file_storage::{clean, PostCommit, Smudge};
use gix::ThreadSafeRepository;
use std::path::PathBuf;

#[derive(Parser)]
enum Command {
    Clean {
        filepath: PathBuf,
        #[arg(default_value_t = ByteSize::mb(50))]
        size: ByteSize,
    },
    Smudge {
        filepath: PathBuf,
    },
    PostCommit,
    // PrePush - find missing refs, push one at a time.
}

#[derive(Parser)]
struct Args {
    #[command(flatten)]
    pub verbosity: Verbosity<InfoLevel>,

    #[command(subcommand)]
    pub command: Command,
}

fn main() -> Result<()> {
    let args = Args::parse();

    env_logger::Builder::new()
        .filter_level(args.verbosity.log_level_filter())
        .try_init()?;

    let repo = ThreadSafeRepository::open(".")?.to_thread_local();

    match args.command {
        // Maybe we should just have functions that do the action,
        // and use the stucts we have to localise state
        Command::Clean { filepath, size } => {
            clean(repo, filepath, size)?;
        }
        Command::Smudge { filepath } => {
            Smudge::new(filepath)?.git_smudge()?;
        }
        Command::PostCommit => {
            PostCommit::new()?.git_post_commit()?;
        }
    };

    Ok(())
}
