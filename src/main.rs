use anyhow::Result;
use bytesize::ByteSize;
use clap::Parser;
use clap_verbosity_flag::{InfoLevel, Verbosity};
use git_file_storage::{Clean, PostCommit, Smudge};
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

    match args.command {
        Command::Clean { filepath, size } => {
            Clean::new(filepath, size)?.git_clean()?;
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
