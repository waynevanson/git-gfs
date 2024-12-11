use anyhow::{anyhow, Result};
use gix::{
    bstr::{BStr, BString, ByteSlice},
    Id, Repository,
};
use itertools::Itertools;
use std::io::{stdin, Read};

// get all the commits within the range (from stdin)
// get list of files for each commit & `.gitattributes`
// filter for big files (where they could have been added)
// read these files (from git objects directly)
// get the reference hash
// push `/refs/gfs/<hash>`, one hash at a time.
pub fn pre_push(repo: &mut Repository) -> Result<()> {
    // get a list of commits that we're going to push.
    // rev_walk?
    let bstr = bstring_from_stdin()?;

    let tips = spec_tips_from_bstr(repo, bstr.as_ref())?;

    let _walk = repo.rev_walk(tips).first_parent_only().all()?;

    todo!("pre-push hook not implemented");
}

fn bstring_from_stdin() -> Result<BString> {
    let mut buf = Vec::with_capacity(64 * 8);
    stdin().read_to_end(&mut buf)?;
    Ok(buf.into())
}

fn spec_tips_from_bstr<'repo>(repo: &'repo Repository, bstr: &BStr) -> Result<[Id<'repo>; 2]> {
    let (from, to) = ids_from_bstr(bstr)?;

    let from = repo.rev_parse_single(from)?;
    let to = repo.rev_parse_single(to)?;

    Ok([from, to])
}

fn ids_from_bstr(bstr: &BStr) -> Result<(&BStr, &BStr)> {
    let (_, from, _, to): (_, _, _, _) = bstr
        .split(|byte| byte == &b' ')
        .map(|byte| byte.as_bstr())
        .collect_tuple()
        .ok_or_else(|| anyhow!("Expected to find a string with 3 spaces in between"))?;

    Ok((from, to))
}
