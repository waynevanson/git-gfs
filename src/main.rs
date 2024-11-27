use anyhow::Result;
use clap::Parser;
use obfuscat::*;

fn main() -> Result<()> {
    println!("Hello, world!");

    let args = Args::parse();

    args.command.call()?;

    Ok(())
}
