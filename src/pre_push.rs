use crate::{create_gfs_ref, Pointer};
use anyhow::{anyhow, Result};
use gix::{
    attrs::{
        parse::{Kind, Lines},
        AssignmentRef,
    },
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
    collections::HashMap, io::{stdin, Read}, ops::Not, process::Command
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

fn get_git_attributes_patterns(entry_ref: EntryRef<'_, '_>) -> Result<HashSet<Pattern>> {
    let data = &entry_ref.object()?.data;

    let lines = gix::attrs::parse(data.as_slice());

    let patterns_attributes = get_pattern_attributes(lines)?;

    let patterns = get_patterns_from_patterns_attributes(patterns_attributes)?;

    Ok(patterns)
}

fn get_patterns_from_patterns_attributes(
    patterns_attributes: HashMap<Pattern, (bool, bool)>,
) -> Result<HashSet<Pattern>> {
    // todo: warn when is_filter_gfs is true but the rest are not.
    let patterns: HashSet<Pattern> = patterns_attributes
        .into_iter()
        .filter(|(_, (is_filter_gfs, negative_text))| *is_filter_gfs && *negative_text)
        .map(|(pattern, _)| pattern)
        .collect();

    Ok(patterns)
}

fn get_pattern_attributes(lines: Lines) -> Result<HashMap<Pattern, (bool, bool)>> {
    // "<pattern> filter=gfs -text"
    let mut patterns_attributes = HashMap::<Pattern, (bool, bool)>::new();

    for line in lines {
        match line? {
            // todo: handle macros
            (Kind::Macro(_), ..) => {}
            (Kind::Pattern(pattern), assignments, ..) => {
                let (is_filter_gfs, negative_text) =
                    patterns_attributes.entry(pattern).or_default();

                if *is_filter_gfs && *negative_text {
                    continue;
                }

                for assignment in assignments {
                    let assignment = assignment?;

                    if is_filter_gfs.not() {
                        *is_filter_gfs = has_gfs_attributes_filter(assignment);
                    }

                    if !*negative_text {
                        *negative_text = is_assignment_negative_text(assignment);
                    }
                }
            }
        };
    }

    Ok(patterns_attributes)
}

fn is_assignment_negative_text(assignment: AssignmentRef<'_>) -> bool {
    assignment.name.as_str() == "-text" && assignment.state.as_bstr().is_none()
}

fn has_gfs_attributes_filter(assignment: AssignmentRef<'_>) -> bool {
    let is_filter = assignment.name.as_str() == "filter";

    let is_gfs = assignment.state.as_bstr().map(|bstr| bstr == "gfs");

    matches!((is_filter, is_gfs), (true, Some(true)))
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
