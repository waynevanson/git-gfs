use anyhow::{anyhow, Result};
use clap::Parser;
use gix::{attrs::glob::Pattern, glob::parse};

#[derive(Parser)]
pub struct Track {
    #[arg(value_parser = pattern_from_str)]
    pattern: Pattern,
}

fn pattern_from_str(str: &str) -> Result<Pattern> {
    let pattern = parse(str).ok_or_else(|| anyhow!("Expected a pattern but received '{}'", str))?;
    Ok(pattern)
}
