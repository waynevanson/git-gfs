use crate::{create_gfs_ref, Pointer};
use anyhow::{anyhow, Result};
use gix::{
    attrs::parse as parse_attributes_from_bytes,
    bstr::{BStr, BString, ByteSlice},
    glob::{wildmatch, Pattern},
    hashtable::hash_set::HashSet,
    object::tree::EntryRef,
    objs::tree::EntryKind,
    traverse::tree::recorder::Entry,
    Id, ObjectId, Repository,
};
use itertools::Itertools;
use std::{
    io::{stdin, Read},
    process::Command,
};

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
    let pushable_references = get_pushable_references(repo, tips)?;

    // push all the tree refs
    // this isn't part of gix yet.
    // we'll have to use git to manage this.
    // git push :hash

    for reference in pushable_references {
        Command::new("git")
            .args(["push", reference.as_str()])
            .status()?;
    }

    Ok(())
}

fn get_pushable_references(repo: &Repository, tips: [Id<'_>; 2]) -> Result<HashSet<String>> {
    let mut pointers = HashSet::<String>::new();

    let infos = repo.rev_walk(tips).all()?;

    for info in infos {
        // get files for each object?
        let tree = info?.object()?.tree()?;

        // no git attributes means no things to do.
        let Some(gitattributes) = tree.find_entry(".gitattributes") else {
            continue;
        };

        let patterns = get_git_attributes_patterns(gitattributes)?;

        // file names
        let files = tree
            .traverse()
            .breadthfirst
            .files()?
            .into_iter()
            .filter(|entry| is_entry_blob_matching_pattern(entry, &patterns))
            .map(|entry| get_pointer_full_reference_pointer_oid(repo, entry.oid));

        for file in files {
            pointers.insert(file?);
        }
    }

    Ok(pointers)
}

fn is_entry_blob_matching_pattern(entry: &Entry, patterns: &HashSet<Pattern>) -> bool {
    entry.mode.kind() == EntryKind::Blob
        && patterns.iter().any(|pattern| {
            pattern.matches(
                entry.filepath.as_ref(),
                wildmatch::Mode::NO_MATCH_SLASH_LITERAL,
            )
        })
}

fn get_pointer_full_reference_pointer_oid(
    repo: &Repository,
    oid: impl Into<ObjectId>,
) -> Result<String> {
    let contents = &repo.find_blob(oid)?.data;
    let pointer: Pointer = serde_json::from_slice(contents.as_slice())?;
    let hash = pointer.hash();
    let reference = create_gfs_ref(hash);
    Ok(reference)
}

fn get_git_attributes_patterns(entry_ref: EntryRef<'_, '_>) -> Result<HashSet<Pattern>> {
    let patterns = HashSet::new();

    let data = &entry_ref.object()?.data;

    // todo: find patterns, including attrs.
    let _lines = parse_attributes_from_bytes(data.as_slice());

    todo!("Bro gotta parse those attributes so we can get patterns so we know what files to get pointers for");

    Ok(patterns)
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
