use anyhow::Result;
use clap::Parser;
use git_file_storage::Args;

fn main() -> Result<()> {
    let args = Args::parse();

    args.command.run()?;

    Ok(())
}
