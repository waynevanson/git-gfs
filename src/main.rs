use anyhow::Result;
use bytesize::ByteSize;
use clap::Parser;
use clap_verbosity_flag::{InfoLevel, Verbosity};
use git_file_storage::{clean, pre_push, smudge};
use gix::ThreadSafeRepository;
use std::path::PathBuf;

#[derive(Parser)]
enum Command {
    /// The integration command used when checking in a file using git.
    ///
    /// Transforms a `filepath` into parts of `size`,
    /// stored as blobs within a tree within a reference under
    /// `refs/gfs/{tree_id}`, the reference in a pointer
    /// send to `stdout` so git can store it as a file.
    Clean {
        filepath: PathBuf,
        #[arg(default_value_t = ByteSize::mb(50))]
        size: ByteSize,
    },
    /// The intergation command used when checking out a file in git.
    Smudge { filepath: PathBuf },
    /// The command used in the `pre-push` hook,
    /// which uploads one reference at a time.
    PrePush,
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

    let mut repo = ThreadSafeRepository::open(".")?.to_thread_local();

    match args.command {
        Command::Clean { filepath, size } => {
            clean(&repo, filepath, size)?;
        }
        Command::Smudge { filepath } => {
            smudge(&repo, filepath)?;
        }
        Command::PrePush => {
            pre_push(&mut repo)?;
        }
    };

    Ok(())
}
