use anyhow::Result;
use clap::Parser;
use git_file_storage::Args;

fn main() -> Result<()> {
    println!("Hello, world!");

    let args = Args::parse();

    args.command.run()?;

    Ok(())
}
