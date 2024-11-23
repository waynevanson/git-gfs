use bytesize::ByteSize;
use clap::Parser;
use clap_verbosity_flag::Verbosity;

#[derive(Parser)]
pub enum Command {
    Bootstrap {},
    Initialise {},
    PostCommit {
        #[arg(short, long)]
        size_limit: ByteSize,
    },
    PrePush {
        #[arg(short, long)]
        size_limit: ByteSize,
    },
}

// When a user pushes and git hooks are on, it should automatically
// automatically push the other commit.
impl Command {
    pub fn call(self) {
        match self {
            _ => unimplemented!(),
        }
    }
}

#[derive(Parser)]
pub struct Args {
    #[command(flatten)]
    verbosity: Verbosity,

    #[command(subcommand)]
    command: Command,
}
