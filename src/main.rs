#![feature(iter_intersperse)]

mod clean;
mod config;
mod content_sha;
mod flat_map_ok;
mod git_object_id;
mod iter_reader_result;
mod smudge;

use crate::{clean::clean, config::Config, smudge::smudge};
use anyhow::Result;
use clap::Parser;
use clap_verbosity_flag::{InfoLevel, Verbosity};
use log::trace;
use serde_jsonc::from_reader;
use std::{fs::File, io::ErrorKind};

#[derive(Parser)]
enum Command {
    /// A git attribute filter to check in large files.
    ///
    /// This does the following:
    /// 1. Replaces the file with a pointer; a list of sha1 hashes.
    /// 2. Splits the file contents (from stdin) to distinct blobs, stored as sha1 hashes in `.gfs/contents/<hash>`.
    ///
    /// For the inverse, please use `git-gfs smudge`.
    Clean,
    /// A git attribute filter to check out large files.
    ///
    /// This does the following:
    /// 1. Reads the pointer (from stdin); a list of sha1 hashes.
    /// 2. Reads and concatenates all the blobs from `.gfs/contents/<hash>`.
    /// 3. Replaces the pointer with the concatenation of blobs.
    ///
    /// For the inverse, please use `git-gfs clean`.
    Smudge,
    // todo: add Check command'
    // check that the filters exist in the git config.
    // check that the pack limit is small
    //
    // test to see if I need prepush to split up a push into multiple.
}

#[derive(Parser)]
struct Args {
    #[command(flatten)]
    pub verbosity: Verbosity<InfoLevel>,

    #[command(subcommand)]
    pub command: Command,
}

fn main() -> Result<()> {
    let args = Args::try_parse()?;

    env_logger::Builder::new()
        .filter_level(args.verbosity.log_level_filter())
        .try_init()?;

    trace!(
        "Initialized logger with {}",
        args.verbosity.log_level_filter()
    );

    let config: Config = match File::open(".gfs/config.jsonc") {
        Ok(file) => {
            trace!("Config exists");
            Ok(from_reader(file)?)
        }
        Err(err) if err.kind() == ErrorKind::NotFound => {
            trace!("Config missing, using defaults");
            Ok(Config::default())
        }
        Err(err) => Err(err),
    }?;

    trace!("Initialized config file");

    match args.command {
        Command::Clean => clean(config.try_into()?)?,
        Command::Smudge => smudge()?,
    }

    Ok(())
}
