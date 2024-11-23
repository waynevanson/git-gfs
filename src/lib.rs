use bytesize::ByteSize;
use clap::Parser;

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

#[derive(Parser)]
pub struct Args {
    #[command(subcommand)]
    command: Command,
}
