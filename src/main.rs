use anyhow::Result;
use bytesize::ByteSize;
use clap::Parser;
use clap_verbosity_flag::{InfoLevel, Verbosity};
use git_file_storage::{clean, pre_push, smudge};
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
