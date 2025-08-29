use crate::flat_map_ok::IntoFlatMapOkIter;
use crate::iter_reader_result::IntoIterReaderResult;
use anyhow::{anyhow, Result};
use itertools::Itertools;
use std::io::{stdin, stdout};
use std::process::{Command, Stdio};

// read a list of git-sha's from the index and output the contents.
pub fn smudge() -> Result<()> {
    let mut git = Command::new("git")
        .arg("--batch-command")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()?;

    // how to get stdin, transform it, handle errors, then continue?

    let mut ins = stdin()
        .lines()
        .map_ok(|git_object_id| {
            format!("contents {}\n", git_object_id)
                .to_string()
                .into_bytes()
                .into_iter()
        })
        .flat_map_ok(|bytes| bytes.map(Ok))
        .into_iter_reader_result();

    let mut git_stdin = git
        .stdin
        .as_mut()
        .ok_or_else(|| anyhow!("Unable to find stdin on git process"))?;

    let mut git_stdout = git
        .stdout
        .as_mut()
        .ok_or_else(|| anyhow!("Unable to find stdout on git process"))?;

    let mut out = stdout();

    std::io::copy(&mut ins, &mut git_stdin)?;
    std::io::copy(&mut git_stdout, &mut out)?;

    git.wait()?;

    Ok(())
}
