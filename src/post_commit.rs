use crate::{map_ok_then::MapOkThen, pointer::Pointer, REFS_NAMESPACE};
use anyhow::{anyhow, Result};
use clap::Parser;
use gix::{
    attrs::search::Match,
    bstr::ByteSlice,
    glob::{wildmatch, Pattern},
    hashtable::HashSet,
    object::tree::EntryRef,
    objs::tree::EntryKind,
    refs::transaction::PreviousValue,
    Commit, Id, Repository,
};
use itertools::Itertools;

#[derive(Parser)]
pub struct PostCommit;

impl PostCommit {
    pub fn run(repo: &mut Repository) -> Result<()> {
        // get branch the commit is on.
        // should always be here since this should be run post commit.
        let commit = repo.head_commit()?;

        // The commit before our commit, where we can merge stuff.
        let Some(parent_id) = get_parent_id(&commit)? else {
            return Ok(());
        };

        // create reference to commit for use later during merge
        let name = format!("{REFS_NAMESPACE}/commits/{}", commit.id());
        let commit_reference_id = repo
            .reference(name, commit.id(), PreviousValue::Any, "")?
            .id();

        // git reset --hard HEAD~1
        let mut branch = repo
            .head()?
            .try_into_referent()
            .ok_or_else(|| anyhow!("Expected HEAD to be attached to a branch"))?;

        branch.set_target_id(parent_id, "")?;

        // how to get all the trees?
        // There's no parent.
        // We could get all those in `refs/gfs/trees/*` and only get those that don't have children?
        // If we end up adding parents to those trees, we'll be able to get the children from the parent that
        // are in our custom refs.

        // git merge branch <commit-reference-id> ...trees
        // TODO: add the parts.
        // BRO read all the pointers from big files in the commit?
        // need to get the filteres lel

        let patterns = get_patterns(repo)?;

        let worktree = repo
            .worktree()
            .ok_or_else(|| anyhow!("Expected to find work tree"))?;

        let tree = commit.tree()?;

        let files = tree
            .iter()
            .filter_ok(is_entry_file)
            .filter_ok(|entry_ref| patterns_contains_entry_ref(&patterns, entry_ref))
            .map(|result| -> Result<_> { Ok(result?) })
            .map_ok_then(|entry_ref| {
                // This assumes that object.data actually contains our file.
                // But that doesn't make sense because it only contains changes.
                //
                // We have the file relative to the the parent tree, but not the full path so we can do the fs::read
                let data = &entry_ref.object()?.data;
                let pointer = toml::from_str::<Pointer>(data.to_str()?)?;
                Ok(pointer.hash)
            });
        // read each file to get a pointer, to get the tree reference to add to the commits.

        let merge_commit_id = repo.merge_base_octopus(vec![commit_reference_id])?;

        Ok(())
    }
}

fn get_parent_id<'repo>(commit: &'repo Commit<'repo>) -> Result<Option<Id<'repo>>> {
    let mut parents = commit.parent_ids();

    match (parents.next(), parents.next()) {
        (Some(parent), None) => Ok(Some(parent)),
        (None, None) => Err(anyhow!("Expected head commit to have a parent")),
        // skip post commit, encountered merge commit (which has 2 or more parents)
        (Some(_), Some(_)) => Ok(None),
        _ => unreachable!("Expected iterator to only produce Some before None"),
    }
}

fn get_patterns(repo: &Repository) -> Result<HashSet<Pattern>> {
    let worktree = repo
        .worktree()
        .ok_or_else(|| anyhow!("Expected to find work tree"))?;

    let attributes = worktree.attributes(None)?.attribute_matches();

    // all patterns contain our custom filter=split -text
    let patterns = attributes
        .iter()
        .filter(has_split_attributes)
        .map(|r#match| r#match.pattern.to_owned())
        .collect::<HashSet<_>>();

    Ok(patterns)
}

fn has_split_attributes(r#match: &Match<'_>) -> bool {
    let is_filter = r#match.assignment.name.as_str() == "filter";

    let is_split = r#match
        .assignment
        .state
        .as_bstr()
        .map(|bstr| bstr == "split");

    matches!((is_filter, is_split), (true, Some(true)))
}

fn is_entry_file(entry: &EntryRef<'_, '_>) -> bool {
    matches!(
        entry.mode().kind(),
        EntryKind::Blob | EntryKind::BlobExecutable
    )
}

fn patterns_contains_entry_ref(patterns: &HashSet<Pattern>, entry_ref: &EntryRef<'_, '_>) -> bool {
    patterns.iter().any(|pattern| {
        pattern.matches(
            entry_ref.filename(),
            wildmatch::Mode::NO_MATCH_SLASH_LITERAL,
        )
    })
}
