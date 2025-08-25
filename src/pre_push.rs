use anyhow::{anyhow, Result};
use bytesize::ByteSize;
use gix::{
    bstr::ByteSlice,
    objs::{FindExt, WriteTo},
    ObjectId, Repository,
};
use itertools::Itertools;
use std::{ffi::OsStr, io::stdin, process::Command, str::FromStr};

// this should come from config
// unique by remote url.
const LIMIT: ByteSize = ByteSize::mb(500);

// get list of commits we need to send.
pub fn pre_push(repo: &mut Repository) -> Result<()> {
    for line in stdin().lines() {
        let line = line?;

        let (_local_ref, local_sha1, _remote_ref, remote_sha1) = line
            .split(" ")
            .collect_tuple::<(&str, &str, &str, &str)>()
            .ok_or_else(|| anyhow!("Expected to be able to split the line into 4 segments"))?;

        let commits = Command::new("git")
            .args(["rev-list", local_sha1, "--not", remote_sha1])
            .output()?;

        let lines = commits.stdout.lines();

        push_packs(repo, lines)?;
    }

    Ok(())
}

fn push_as_pack(start: Option<&[u8]>, end: Option<&[u8]>) -> Result<()> {
    let args = [start, end]
        .iter()
        .flatten()
        .map(|a| a.to_os_str())
        .collect::<Result<Vec<&OsStr>, _>>()?;

    if args.len() > 0 {
        Command::new("git").arg("send-pack").args(args).output()?;
    }

    Ok(())
}

/// Push packs in sections that are below the allowed threshold of a provider.
/// We assume this to be around 100MB safety.
fn push_packs<'repo, 'a>(
    repo: &Repository,
    mut lines: impl Iterator<Item = &'a [u8]>,
) -> Result<()> {
    let mut start = None;
    let mut end = None;
    let mut total_size = 0;

    let mut line = lines.next();
    while let Some(commit) = line {
        let size = repo
            .objects
            .find_commit(&ObjectId::from_str(commit.to_str()?)?, &mut Vec::new())?
            .size();

        let peeked_total_size = total_size + size;

        if peeked_total_size < LIMIT.as_u64() {
            // go next
            total_size = peeked_total_size;

            if start.is_none() {
                start = Some(commit);
            } else {
                end = Some(commit);
            }
        } else {
            // consume
            push_as_pack(start, end)?;

            // reset
            start = None;
            end = None;
            total_size = 0;
        }

        line = lines.next();
    }

    push_as_pack(start, end)?;

    Ok(())
}
