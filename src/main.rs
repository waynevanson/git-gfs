use anyhow::Result;
use clap::Parser;
use obfuscat::Args;

fn main() -> Result<()> {
    println!("Hello, world!");

    let args = Args::parse();

    args.command.run()?;

    Ok(())
}
